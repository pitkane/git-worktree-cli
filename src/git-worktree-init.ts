#!/usr/bin/env tsx

import { basename, join } from "node:path";
import { writeFile } from "node:fs/promises";
import { $ } from "zx";
import * as yaml from "yaml";

$.verbose = false;

interface GitWorktreeConfig {
	repositoryUrl: string;
	mainBranch: string;
	createdAt: string;
}

async function gwtinit(repoUrl: string) {
	if (!repoUrl) {
		console.error("Error: Repository URL is required");
		console.error("Usage: gwtinit <repository-url>");
		process.exit(1);
	}

	try {
		const repoName = basename(repoUrl, ".git");
		const projectRoot = process.cwd();
		
		console.log(`Cloning ${repoUrl}...`);
		await $`git clone ${repoUrl} ${repoName}`;
		
		const defaultBranch = await $`cd ${repoName} && git symbolic-ref --short HEAD`.text();
		const branchName = defaultBranch.trim();
		
		const finalDirName = branchName;
		
		await $`mv ${repoName} ${finalDirName}`;
		
		// Create configuration file in project root
		const config: GitWorktreeConfig = {
			repositoryUrl: repoUrl,
			mainBranch: branchName,
			createdAt: new Date().toISOString()
		};
		
		const configPath = join(projectRoot, "git-worktree-config.yaml");
		const configContent = yaml.stringify(config);
		await writeFile(configPath, configContent, "utf-8");
		
		console.log(`✓ Repository cloned to: ${finalDirName}`);
		console.log(`✓ Default branch: ${branchName}`);
		console.log(`✓ Config saved to: ${configPath}`);
		
	} catch (error) {
		console.error("Error:", error instanceof Error ? error.message : String(error));
		process.exit(1);
	}
}

const repoUrl = process.argv[2];
await gwtinit(repoUrl);