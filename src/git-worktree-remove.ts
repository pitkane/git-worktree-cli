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
	
	return worktrees;
}

async function getCurrentWorktreePath(): Promise<string | null> {
	try {
		const gitRoot = await $`git rev-parse --show-toplevel 2>/dev/null`.text();
		return gitRoot.trim();
	} catch {
		return null;
	}
}

async function findWorktreeToRemove(branchName: string, worktrees: Worktree[]): Promise<Worktree | null> {
	// Try to find worktree by branch name
	let targetWorktree = worktrees.find(wt => {
		const cleanBranch = wt.branch ? wt.branch.replace('refs/heads/', '') : '';
		return cleanBranch === branchName;
	});
	
	// If not found by branch name, try by path (last part of path)
	if (!targetWorktree) {
		targetWorktree = worktrees.find(wt => {
			const pathParts = wt.path.split('/');
			const lastPart = pathParts[pathParts.length - 1];
			return lastPart === branchName;
		});
	}
	
	return targetWorktree || null;
}

async function gwtremove(branchName?: string) {
	try {
		const worktrees = await getAllWorktrees();
		
		if (worktrees.length === 0) {
			console.error("No worktrees found.");
			process.exit(1);
		}
		
		let targetWorktree: Worktree | null = null;
		
		if (!branchName) {
			// No parameter given, remove current worktree
			const currentPath = await getCurrentWorktreePath();
			if (!currentPath) {
				console.error("Error: Not in a git worktree. Please specify a branch to remove.");
				process.exit(1);
			}
			
			targetWorktree = worktrees.find(wt => wt.path === currentPath) || null;
			if (!targetWorktree) {
				console.error("Error: Current directory is not a known worktree.");
				process.exit(1);
			}
		} else {
			// Parameter given, find the specified worktree
			targetWorktree = await findWorktreeToRemove(branchName, worktrees);
			if (!targetWorktree) {
				console.error(`Error: Worktree for '${branchName}' not found.`);
				console.log("\nAvailable worktrees:");
				for (const worktree of worktrees) {
					const cleanBranch = worktree.branch ? worktree.branch.replace('refs/heads/', '') : worktree.HEAD.slice(0, 7);
					console.log(`  ${cleanBranch} -> ${worktree.path}`);
				}
				process.exit(1);
			}
		}
		
		// Check if this is the main/bare repository
		if (targetWorktree.bare) {
			console.error("Error: Cannot remove the main (bare) repository.");
			process.exit(1);
		}
		
		const cleanBranch = targetWorktree.branch ? targetWorktree.branch.replace('refs/heads/', '') : targetWorktree.HEAD.slice(0, 7);
		
		// Confirm removal
		console.log(`About to remove worktree:`);
		console.log(`  Path: ${targetWorktree.path}`);
		console.log(`  Branch: ${cleanBranch}`);
		
		// Check if we're currently in the worktree being removed
		const currentPath = await getCurrentWorktreePath();
		const willRemoveCurrent = currentPath === targetWorktree.path;
		
		if (willRemoveCurrent) {
			console.log("\n⚠️  You are currently in this worktree. You will be moved to the project root after removal.");
		}
		
		// Ask for confirmation
		console.log("");
		const confirmation = await new Promise<string>((resolve) => {
			process.stdout.write("Are you sure you want to remove this worktree? (y/N): ");
			process.stdin.setEncoding('utf8');
			process.stdin.once('data', (data) => {
				// Clean up stdin immediately
				process.stdin.pause();
				resolve(data.toString().trim());
			});
		});
		
		if (confirmation.toLowerCase() !== 'y' && confirmation.toLowerCase() !== 'yes') {
			console.log("Removal cancelled.");
			process.exit(0);
		}
		
		// Find the main branch worktree to run commands from
		const mainBranches = ['main', 'master', 'dev', 'develop'];
		let mainBranchWorktree = worktrees.find(wt => {
			const branchName = wt.branch ? wt.branch.replace('refs/heads/', '') : '';
			return mainBranches.includes(branchName);
		});
		
		// If no main branch found, use any other worktree
		if (!mainBranchWorktree) {
			mainBranchWorktree = worktrees.find(wt => wt.path !== targetWorktree!.path);
		}
		
		if (!mainBranchWorktree) {
			throw new Error("No other worktrees found to execute git command from.");
		}
		
		const gitWorkingDir = mainBranchWorktree.path;
		
		// Change to main branch worktree to perform operations
		const originalCwd = process.cwd();
		process.chdir(gitWorkingDir);
		
		try {
			// Remove the worktree
			console.log("\nRemoving worktree...");
			await $`git worktree remove ${targetWorktree.path} --force`;
			
			// Also delete the branch if it's not a main branch
			if (!mainBranches.includes(cleanBranch)) {
				try {
					await $`git branch -D ${cleanBranch}`;
					console.log(`✓ Worktree removed: ${targetWorktree.path}`);
					console.log(`✓ Branch deleted: ${cleanBranch}`);
				} catch (error) {
					// Branch deletion failed, but worktree was removed
					console.log(`✓ Worktree removed: ${targetWorktree.path}`);
					console.log(`⚠️  Branch '${cleanBranch}' could not be deleted automatically`);
					console.log(`   (Error: ${error instanceof Error ? error.message : String(error)})`);
				}
			} else {
				console.log(`✓ Worktree removed: ${targetWorktree.path}`);
				console.log(`✓ Branch: ${cleanBranch} (preserved - main branch)`);
			}
		} finally {
			// Always return to original directory
			process.chdir(originalCwd);
		}
		
	} catch (error) {
		console.error("Error:", error instanceof Error ? error.message : String(error));
		process.exit(1);
	}
}

const branchName = process.argv[2];
await gwtremove(branchName);