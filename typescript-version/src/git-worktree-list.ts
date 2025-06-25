#!/usr/bin/env tsx

import { $ } from "zx";
import { existsSync } from "node:fs";
import { join } from "node:path";
import { findWorktreesFromExisting, parseWorktreeList, type Worktree, cleanBranchName } from "./git-worktree-utils.js";

$.verbose = false;

async function gwtlist() {
	try {
		let worktrees: Worktree[] = [];
		
		// Check if we're in a git repository
		try {
			const output = await $`git worktree list --porcelain 2>/dev/null`.text();
			worktrees = parseWorktreeList(output);
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
			const cleanBranch = cleanBranchName(wt.branch) || wt.HEAD.slice(0, 7);
			
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