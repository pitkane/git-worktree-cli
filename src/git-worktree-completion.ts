#!/usr/bin/env tsx

import { join } from "node:path";
import { existsSync } from "node:fs";
import { readdir } from "node:fs/promises";
import { $ } from "zx";

$.verbose = false;

interface Worktree {
	path: string;
	HEAD: string;
	branch: string;
	bare?: boolean;
}

async function findWorktreesFromExisting(rootPath: string): Promise<Worktree[]> {
	// Find any existing worktree to use for git commands
	const entries = await readdir(rootPath, { withFileTypes: true });
	let foundWorktree: string | null = null;
	
	// First try direct subdirectories
	for (const entry of entries) {
		if (entry.isDirectory()) {
			const dirPath = join(rootPath, entry.name);
			const gitPath = join(dirPath, ".git");
			if (existsSync(gitPath)) {
				foundWorktree = dirPath;
				break;
			}
		}
	}
	
	// If not found in direct subdirectories, try one level deeper
	if (!foundWorktree) {
		for (const entry of entries) {
			if (entry.isDirectory()) {
				const subDir = join(rootPath, entry.name);
				try {
					const subEntries = await readdir(subDir, { withFileTypes: true });
					for (const subEntry of subEntries) {
						if (subEntry.isDirectory()) {
							const dirPath = join(subDir, subEntry.name);
							const gitPath = join(dirPath, ".git");
							if (existsSync(gitPath)) {
								foundWorktree = dirPath;
								break;
							}
						}
					}
					if (foundWorktree) break;
				} catch {
					// Skip if we can't read subdirectory
				}
			}
		}
	}
	
	if (!foundWorktree) {
		return [];
	}
	
	// Use the found worktree to get the complete list via git
	try {
		const output = await $`cd ${foundWorktree} && git worktree list --porcelain`.text();
		const worktrees: Worktree[] = [];
		let currentWorktree: Partial<Worktree> = {};
		
		for (const line of output.split('\n')) {
			if (line.startsWith('worktree ')) {
				if (currentWorktree.path) {
					worktrees.push(currentWorktree as Worktree);
				}
				currentWorktree = { path: line.slice(9) };
			} else if (line.startsWith('HEAD ')) {
				currentWorktree.HEAD = line.slice(5);
			} else if (line.startsWith('branch ')) {
				currentWorktree.branch = line.slice(7);
			} else if (line === 'bare') {
				currentWorktree.bare = true;
			}
		}
		
		if (currentWorktree.path) {
			worktrees.push(currentWorktree as Worktree);
		}
		
		return worktrees;
	} catch {
		return [];
	}
}

async function getAllWorktrees(): Promise<Worktree[]> {
	try {
		let worktrees: Worktree[] = [];
		
		// Check if we're in a git repository
		try {
			const output = await $`git worktree list --porcelain 2>/dev/null`.text();
			let currentWorktree: Partial<Worktree> = {};
			
			for (const line of output.split('\n')) {
				if (line.startsWith('worktree ')) {
					if (currentWorktree.path) {
						worktrees.push(currentWorktree as Worktree);
					}
					currentWorktree = { path: line.slice(9) };
				} else if (line.startsWith('HEAD ')) {
					currentWorktree.HEAD = line.slice(5);
				} else if (line.startsWith('branch ')) {
					currentWorktree.branch = line.slice(7);
				} else if (line === 'bare') {
					currentWorktree.bare = true;
				}
			}
			
			if (currentWorktree.path) {
				worktrees.push(currentWorktree as Worktree);
			}
		} catch {
			// Not in a git repository, check if we're in a project root
			const configPath = join(process.cwd(), "git-worktree-config.yaml");
			if (existsSync(configPath)) {
				// We're in a project root, scan for worktrees
				worktrees = await findWorktreesFromExisting(process.cwd());
			}
		}
		
		return worktrees;
	} catch {
		return [];
	}
}

async function getAvailableBranches(command: string): Promise<string[]> {
	const worktrees = await getAllWorktrees();
	
	if (worktrees.length === 0) {
		return [];
	}
	
	if (command === "gwtswitch" || command === "gwtremove") {
		// For gwtswitch and gwtremove, return existing worktree branches
		return worktrees
			.map(wt => wt.branch ? wt.branch.replace('refs/heads/', '') : '')
			.filter(branch => branch.length > 0);
	} else if (command === "gwtadd") {
		// For gwtadd, return available remote branches (not implemented yet)
		// This would require querying remote branches
		return [];
	}
	
	return [];
}

async function completion() {
	const args = process.argv.slice(2);
	const command = args[0]; // gwtswitch, gwtadd, or gwtremove
	const currentWord = args[1] || ""; // Current word being completed
	
	try {
		const branches = await getAvailableBranches(command);
		
		if (branches.length === 0) {
			if (command === "gwtswitch") {
				// No worktrees found - output a helpful message
				console.log("# No worktrees found. Use 'gwtinit' to create one first.");
			} else if (command === "gwtadd") {
				console.log("# Tab completion for gwtadd not implemented yet.");
			}
			return;
		}
		
		// Filter branches that start with the current word
		const matches = branches.filter(branch => 
			branch.toLowerCase().startsWith(currentWord.toLowerCase())
		);
		
		// Output matches for shell completion
		for (const match of matches) {
			console.log(match);
		}
		
	} catch (_error) {
		// Silently fail for tab completion
	}
}

await completion();