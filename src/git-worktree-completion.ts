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

async function findWorktrees(rootPath: string): Promise<Worktree[]> {
	const worktrees: Worktree[] = [];
	const entries = await readdir(rootPath, { withFileTypes: true });
	
	for (const entry of entries) {
		if (entry.isDirectory()) {
			const dirPath = join(rootPath, entry.name);
			const gitPath = join(dirPath, ".git");
			
			if (existsSync(gitPath)) {
				try {
					// Get branch info from this worktree
					const branch = await $`cd ${dirPath} && git branch --show-current`.text();
					const head = await $`cd ${dirPath} && git rev-parse HEAD`.text();
					
					worktrees.push({
						path: dirPath,
						branch: branch.trim() ? `refs/heads/${branch.trim()}` : "",
						HEAD: head.trim(),
						bare: false
					});
				} catch {
					// Skip if we can't get git info
				}
			}
		}
	}
	
	return worktrees;
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
				worktrees = await findWorktrees(process.cwd());
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
	
	if (command === "gwtswitch") {
		// For gwtswitch, return existing worktree branches
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
	const command = args[0]; // gwtswitch or gwtadd
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