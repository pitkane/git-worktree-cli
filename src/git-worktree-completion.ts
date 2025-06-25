#!/usr/bin/env tsx

import { getAllWorktrees, Worktree, cleanBranchName } from "./git-worktree-utils.js";

async function getAvailableBranches(command: string): Promise<string[]> {
	const worktrees = await getAllWorktrees();
	
	if (worktrees.length === 0) {
		return [];
	}
	
	if (command === "gwtswitch" || command === "gwtremove") {
		// For gwtswitch and gwtremove, return existing worktree branches
		return worktrees
			.map(wt => cleanBranchName(wt.branch))
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