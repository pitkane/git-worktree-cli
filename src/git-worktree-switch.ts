#!/usr/bin/env tsx

import { $ } from "zx";
import { getAllWorktrees, Worktree, cleanBranchName } from "./git-worktree-utils.js";

$.verbose = false;

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
				const cleanBranch = cleanBranchName(worktree.branch) || worktree.HEAD.slice(0, 7);
				const isBare = worktree.bare ? ' (bare)' : '';
				console.log(`  ${cleanBranch}${isBare}`);
			}
			
			console.log("\nUsage: gwtswitch <branch-name>");
			return;
		}
		
		// Find the worktree for the specified branch
		const targetWorktree = worktrees.find(wt => {
			const cleanBranch = cleanBranchName(wt.branch);
			return cleanBranch === branchName;
		});
		
		if (!targetWorktree) {
			console.error(`Error: Worktree for branch '${branchName}' not found.`);
			console.log("\nAvailable branches:");
			for (const worktree of worktrees) {
				const cleanBranch = cleanBranchName(worktree.branch) || worktree.HEAD.slice(0, 7);
				console.log(`  ${cleanBranch}`);
			}
			process.exit(1);
		}
		
		// Switch to the target worktree
		console.log(`Switching to worktree: ${targetWorktree.path}`);
		
	} catch (error) {
		console.error("Error:", error instanceof Error ? error.message : String(error));
		process.exit(1);
	}
}

const branchName = process.argv[2];
await gwtswitch(branchName);