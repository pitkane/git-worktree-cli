#!/usr/bin/env tsx

import { basename, join } from "node:path";
import { existsSync } from "node:fs";
import { readFile, readdir } from "node:fs/promises";
import { $ } from "zx";
import * as yaml from "yaml";

$.verbose = false;

interface GitWorktreeConfig {
	repositoryUrl: string;
	mainBranch: string;
	createdAt: string;
}

async function getMainBranch(gitRoot?: string): Promise<string> {
	try {
		// Try to read from config file first (when in project root)
		const configPath = join(process.cwd(), "git-worktree-config.yaml");
		if (existsSync(configPath)) {
			const configContent = await readFile(configPath, "utf-8");
			const config: GitWorktreeConfig = yaml.parse(configContent);
			return config.mainBranch;
		}

		// If we have a git root, try git commands from there
		if (gitRoot) {
			try {
				const remoteBranches = await $`cd ${gitRoot} && git branch -r --points-at refs/remotes/origin/HEAD 2>/dev/null`.text();
				if (remoteBranches.trim()) {
					const match = remoteBranches.match(/origin\/(.+)/);
					if (match) {
						return match[1].trim();
					}
				}
			} catch {
				// Try symbolic-ref method
				try {
					const symbolicRef = await $`cd ${gitRoot} && git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null`.text();
					const match = symbolicRef.match(/refs\/remotes\/origin\/(.+)/);
					if (match) {
						return match[1].trim();
					}
				} catch {
					// Continue to fallback
				}
			}
		}

		// Default fallback
		return "main";
	} catch {
		return "main";
	}
}

async function branchExists(branchName: string, gitRoot?: string): Promise<boolean> {
	try {
		const gitCmd = gitRoot ? `cd ${gitRoot} && ` : "";
		
		// Check if branch exists locally
		const localBranches = await $`${gitCmd}git branch --list ${branchName} 2>/dev/null`.text();
		if (localBranches.trim()) {
			return true;
		}

		// Check if branch exists remotely
		const remoteBranches = await $`${gitCmd}git branch -r --list "origin/${branchName}" 2>/dev/null`.text();
		return remoteBranches.trim().length > 0;
	} catch {
		return false;
	}
}

async function findGitRoot(): Promise<string | null> {
	try {
		const gitRoot = await $`git rev-parse --show-toplevel 2>/dev/null`.text();
		return gitRoot.trim();
	} catch {
		return null;
	}
}

async function gwtadd(folderName: string) {
	if (!folderName) {
		console.error("Error: Folder name is required");
		console.error("Usage: gwtadd <folder-name>");
		process.exit(1);
	}

	try {
		const branchName = basename(folderName);
		
		// Determine git root and target path
		let gitRoot: string | null = null;
		let targetPath: string;
		let gitWorkingDir: string;
		
		// Check if we're in a git repository
		gitRoot = await findGitRoot();
		
		if (gitRoot) {
			// We're in a git repository, create worktree relative to git root
			targetPath = join(gitRoot, "..", folderName);
			gitWorkingDir = gitRoot;
		} else {
			// We're in project root, look for existing worktree to use as git repository
			const configPath = join(process.cwd(), "git-worktree-config.yaml");
			if (!existsSync(configPath)) {
				throw new Error("Not in a git repository or project root with git-worktree-config.yaml");
			}
			
			// Find an existing worktree to use for git commands
			const entries = await readdir(process.cwd(), { withFileTypes: true });
			let foundWorktree: string | null = null;
			
			for (const entry of entries) {
				if (entry.isDirectory()) {
					const dirPath = join(process.cwd(), entry.name);
					const gitPath = join(dirPath, ".git");
					if (existsSync(gitPath)) {
						foundWorktree = dirPath;
						break;
					}
				}
			}
			
			if (!foundWorktree) {
				throw new Error("No existing worktrees found. Create one first using gwtinit.");
			}
			
			gitWorkingDir = foundWorktree;
			targetPath = join(process.cwd(), folderName);
		}

		const mainBranch = await getMainBranch(gitWorkingDir);
		const exists = await branchExists(branchName, gitWorkingDir);

		if (exists) {
			console.log(`Branch '${branchName}' already exists, checking out existing branch...`);
			await $`cd ${gitWorkingDir} && git worktree add ${targetPath} ${branchName}`;
		} else {
			console.log(`Creating new branch '${branchName}' from '${mainBranch}'...`);
			await $`cd ${gitWorkingDir} && git worktree add ${targetPath} -b ${branchName} ${mainBranch}`;
		}

		console.log(`✓ Worktree created at: ${targetPath}`);
		console.log(`✓ Branch: ${branchName}`);
		
		// Navigate to the new worktree directory
		console.log(`\nChanging to worktree directory...`);
		process.chdir(targetPath);
		console.log(`✓ Now in: ${process.cwd()}`);

	} catch (error) {
		console.error("Error:", error instanceof Error ? error.message : String(error));
		process.exit(1);
	}
}

const folderName = process.argv[2];
await gwtadd(folderName);