#!/usr/bin/env tsx

import { basename, join } from "node:path";
import { writeFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { $ } from "zx";
import * as yaml from "yaml";
import { executeHooks, type GitWorktreeConfig } from "./git-worktree-utils.js";

$.verbose = false;


async function gwtinit(repoUrl: string) {
	if (!repoUrl) {
		console.error("Error: Repository URL is required");
		console.error("Usage: gwtinit <repository-url>");
		process.exit(1);
	}

	try {
		const repoName = basename(repoUrl, ".git");
		const projectRoot = process.cwd();
		
		// Remove existing clone directory if it exists
		if (existsSync(repoName)) {
			await $`rm -rf ${repoName}`;
		}
		
		console.log(`Cloning ${repoUrl}...`);
		$.verbose = true;
		await $`git clone ${repoUrl} ${repoName}`;
		$.verbose = false;
		
		const defaultBranch = await $`cd ${repoName} && git symbolic-ref --short HEAD`.text();
		const branchName = defaultBranch.trim();
		
		const finalDirName = branchName;
		
		// Remove existing directory if it exists
		if (existsSync(finalDirName)) {
			await $`rm -rf ${finalDirName}`;
		}
		
		await $`mv ${repoName} ${finalDirName}`;
		
		// Create configuration file in project root
		const config: GitWorktreeConfig = {
			repositoryUrl: repoUrl,
			mainBranch: branchName,
			createdAt: new Date().toISOString(),
			hooks: {
				postAdd: [
					"# npm install"
				],
				postSwitch: [
					"# echo 'Switched to branch ${branchName}'"
				],
				postRemove: [
					"# echo 'Removed worktree for branch ${branchName}'"
				],
				postInit: [
					"# echo 'Initialized git worktree project'"
				]
			}
		};
		
		const configPath = join(projectRoot, "git-worktree-config.yaml");
		const configContent = yaml.stringify(config);
		await writeFile(configPath, configContent, "utf-8");
		
		console.log(`✓ Repository cloned to: ${finalDirName}`);
		console.log(`✓ Default branch: ${branchName}`);
		console.log(`✓ Config saved to: ${configPath}`);

		// Execute post-init hooks
		await executeHooks("postInit", join(projectRoot, finalDirName), {
			branchName: branchName,
			worktreePath: join(projectRoot, finalDirName)
		});
		
	} catch (error) {
		console.error("Error:", error instanceof Error ? error.message : String(error));
		process.exit(1);
	}
}

const repoUrl = process.argv[2];
await gwtinit(repoUrl);