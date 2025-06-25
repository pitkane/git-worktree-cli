#!/usr/bin/env tsx

import { $ } from "zx";
import { existsSync } from "node:fs";
import { join } from "node:path";
import { readdir } from "node:fs/promises";

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

async function gwtlist() {
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
			} else {
				throw new Error("Not in a git repository or project root with git-worktree-config.yaml");
			}
		}
		
		// Calculate max lengths for proper column alignment
		let maxPathLength = 4;   // "PATH"
		let maxBranchLength = 6; // "BRANCH"
		
		const worktreeData = worktrees.map(wt => {
			// Show absolute path for clarity
			const displayPath = wt.path;
			// Clean branch name (remove refs/heads/ prefix)
			const cleanBranch = wt.branch ? wt.branch.replace('refs/heads/', '') : wt.HEAD.slice(0, 7);
			
			maxPathLength = Math.max(maxPathLength, displayPath.length);
			maxBranchLength = Math.max(maxBranchLength, cleanBranch.length);
			
			return {
				...wt,
				displayPath,
				cleanBranch
			};
		});
		
		console.log(`\nWorktrees (${worktrees.length}):\n`);
		console.log(`${'PATH'.padEnd(maxPathLength)}  ${'BRANCH'.padEnd(maxBranchLength)}`);
		console.log(`${'─'.repeat(maxPathLength)}  ${'─'.repeat(maxBranchLength)}`);
		
		for (const worktree of worktreeData) {
			const isBare = worktree.bare ? ' (bare)' : '';
			console.log(`${worktree.displayPath.padEnd(maxPathLength)}  ${worktree.cleanBranch}${isBare}`);
		}
		
	} catch (error) {
		console.error("Error:", error instanceof Error ? error.message : String(error));
		process.exit(1);
	}
}

await gwtlist();