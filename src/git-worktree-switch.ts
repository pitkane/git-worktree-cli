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

async function _findGitRoot(): Promise<string | null> {
	try {
		const gitRoot = await $`git rev-parse --show-toplevel 2>/dev/null`.text();
		return gitRoot.trim();
	} catch {
		return null;
	}
}

async function getAllWorktrees(): Promise<Worktree[]> {
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
		} else {
			throw new Error("Not in a git repository or project root with git-worktree-config.yaml");
		}
	}
	
	return worktrees;
}

async function gwtswitch(branchName?: string) {
	try {
		const worktrees = await getAllWorktrees();
		
		if (!branchName) {
			// No branch specified, show available worktrees
			if (worktrees.length === 0) {
				console.log("No worktrees found.");
				return;
			}
			
			console.log("\nAvailable worktrees:");
			console.log("────────────────────");
			
			for (const worktree of worktrees) {
				const cleanBranch = worktree.branch ? worktree.branch.replace('refs/heads/', '') : worktree.HEAD.slice(0, 7);
				const isBare = worktree.bare ? ' (bare)' : '';
				console.log(`  ${cleanBranch}${isBare}`);
			}
			
			console.log("\nUsage: gwtswitch <branch-name>");
			return;
		}
		
		// Find the worktree for the specified branch
		const targetWorktree = worktrees.find(wt => {
			const cleanBranch = wt.branch ? wt.branch.replace('refs/heads/', '') : '';
			return cleanBranch === branchName;
		});
		
		if (!targetWorktree) {
			console.error(`Error: Worktree for branch '${branchName}' not found.`);
			console.log("\nAvailable branches:");
			for (const worktree of worktrees) {
				const cleanBranch = worktree.branch ? worktree.branch.replace('refs/heads/', '') : worktree.HEAD.slice(0, 7);
				console.log(`  ${cleanBranch}`);
			}
			process.exit(1);
		}
		
		// Switch to the target worktree
		console.log(`Switching to worktree: ${targetWorktree.path}`);
		process.chdir(targetWorktree.path);
		console.log(`✓ Now in: ${process.cwd()}`);
		
	} catch (error) {
		console.error("Error:", error instanceof Error ? error.message : String(error));
		process.exit(1);
	}
}

const branchName = process.argv[2];
await gwtswitch(branchName);