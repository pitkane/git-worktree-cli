#!/usr/bin/env tsx

import { basename } from "node:path";
import { $ } from "zx";

$.verbose = false;

async function gwtinit(repoUrl: string) {
	if (!repoUrl) {
		console.error("Error: Repository URL is required");
		console.error("Usage: gwtinit <repository-url>");
		process.exit(1);
	}

	try {
		const repoName = basename(repoUrl, ".git");
		
		console.log(`Cloning ${repoUrl}...`);
		await $`git clone ${repoUrl} ${repoName}`;
		
		const defaultBranch = await $`cd ${repoName} && git symbolic-ref --short HEAD`.text();
		const branchName = defaultBranch.trim();
		
		const finalDirName = branchName;
		
		await $`mv ${repoName} ${finalDirName}`;
		
		console.log(`✓ Repository cloned to: ${finalDirName}`);
		console.log(`✓ Default branch: ${branchName}`);
		
	} catch (error) {
		console.error("Error:", error instanceof Error ? error.message : String(error));
		process.exit(1);
	}
}

const repoUrl = process.argv[2];
await gwtinit(repoUrl);