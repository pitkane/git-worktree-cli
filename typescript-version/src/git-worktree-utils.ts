import { join } from "node:path";
import { existsSync } from "node:fs";
import { readFile, readdir } from "node:fs/promises";
import { $ } from "zx";
import * as yaml from "yaml";

$.verbose = false;

export interface GitWorktreeConfig {
	repositoryUrl: string;
	mainBranch: string;
	createdAt: string;
	hooks?: {
		postAdd?: string[];
		postSwitch?: string[];
		postRemove?: string[];
		postInit?: string[];
	};
}

export interface Worktree {
	path: string;
	HEAD: string;
	branch: string;
	bare?: boolean;
}

export async function findGitRoot(): Promise<string | null> {
	try {
		const gitRoot = await $`git rev-parse --show-toplevel 2>/dev/null`.text();
		return gitRoot.trim();
	} catch {
		return null;
	}
}

export async function getMainBranch(gitRoot?: string): Promise<string> {
	try {
		// First, try to find the config file in current directory or project root
		let configPath = join(process.cwd(), "git-worktree-config.yaml");

		// If not found in current directory and we have gitRoot, try looking in the parent of gitRoot
		if (!existsSync(configPath) && gitRoot) {
			const projectRoot = join(gitRoot, "..");
			configPath = join(projectRoot, "git-worktree-config.yaml");
		}

		if (existsSync(configPath)) {
			const configContent = await readFile(configPath, "utf-8");
			const config: GitWorktreeConfig = yaml.parse(configContent);
			return config.mainBranch;
		}

		// If we have a git root, try git commands from there
		if (gitRoot) {
			try {
				const symbolicRef =
					await $`cd ${gitRoot} && git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null`.text();
				const match = symbolicRef.match(/refs\/remotes\/origin\/(.+)/);
				if (match) {
					return match[1].trim();
				}
			} catch {
				// Try alternative method
				try {
					const remoteBranches =
						await $`cd ${gitRoot} && git branch -r --points-at refs/remotes/origin/HEAD 2>/dev/null`.text();
					if (remoteBranches.trim()) {
						// Parse output like "  origin/dev -> origin/dev" or "  origin/dev"
						const match = remoteBranches.match(/origin\/([^\s-]+)/);
						if (match) {
							return match[1].trim();
						}
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

export async function branchExists(
	branchName: string,
	gitRoot?: string
): Promise<{ exists: boolean; local: boolean; remote: boolean }> {
	try {
		let localBranches: string;
		let remoteBranches: string;

		if (gitRoot) {
			// Execute commands from the git root directory
			localBranches = await $`cd ${gitRoot} && git branch --list ${branchName} 2>/dev/null`.text();
			remoteBranches = await $`cd ${gitRoot} && git branch -r --list "origin/${branchName}" 2>/dev/null`.text();
		} else {
			// Execute commands from current directory
			localBranches = await $`git branch --list ${branchName} 2>/dev/null`.text();
			remoteBranches = await $`git branch -r --list "origin/${branchName}" 2>/dev/null`.text();
		}

		const localExists = localBranches.trim().length > 0;
		const remoteExists = remoteBranches.trim().length > 0;

		return {
			exists: localExists || remoteExists,
			local: localExists,
			remote: remoteExists,
		};
	} catch {
		return { exists: false, local: false, remote: false };
	}
}

export async function findWorktreesFromExisting(rootPath: string): Promise<Worktree[]> {
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
		return parseWorktreeList(output);
	} catch {
		return [];
	}
}

export function parseWorktreeList(output: string): Worktree[] {
	const worktrees: Worktree[] = [];
	let currentWorktree: Partial<Worktree> = {};

	for (const line of output.split("\n")) {
		if (line.startsWith("worktree ")) {
			if (currentWorktree.path) {
				worktrees.push(currentWorktree as Worktree);
			}
			currentWorktree = { path: line.slice(9) };
		} else if (line.startsWith("HEAD ")) {
			currentWorktree.HEAD = line.slice(5);
		} else if (line.startsWith("branch ")) {
			currentWorktree.branch = line.slice(7);
		} else if (line === "bare") {
			currentWorktree.bare = true;
		}
	}

	if (currentWorktree.path) {
		worktrees.push(currentWorktree as Worktree);
	}

	return worktrees;
}

export async function getAllWorktrees(): Promise<Worktree[]> {
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
			throw new Error(
				"Not in a git repository or project root with git-worktree-config.yaml"
			);
		}
	}

	return worktrees;
}

export function cleanBranchName(branch: string): string {
	return branch ? branch.replace("refs/heads/", "") : "";
}

export async function getCurrentWorktreePath(): Promise<string | null> {
	try {
		const gitRoot = await $`git rev-parse --show-toplevel 2>/dev/null`.text();
		return gitRoot.trim();
	} catch {
		return null;
	}
}

export async function executeHooks(
	hookType: "postAdd" | "postSwitch" | "postRemove" | "postInit",
	workingDirectory: string,
	context?: { branchName?: string; worktreePath?: string }
): Promise<void> {
	try {
		// Find the config file
		let configPath = join(process.cwd(), "git-worktree-config.yaml");
		
		// If not found in current directory, try looking in the parent directories
		let currentDir = workingDirectory;
		while (currentDir !== "/" && currentDir !== ".") {
			const testConfigPath = join(currentDir, "git-worktree-config.yaml");
			if (existsSync(testConfigPath)) {
				configPath = testConfigPath;
				break;
			}
			currentDir = join(currentDir, "..");
		}

		if (!existsSync(configPath)) {
			// No config file found, skip hooks
			return;
		}

		const configContent = await readFile(configPath, "utf-8");
		const config: GitWorktreeConfig = yaml.parse(configContent);

		if (!config.hooks || !config.hooks[hookType]) {
			// No hooks defined for this type
			return;
		}

		const hooks = config.hooks[hookType];
		if (!hooks || hooks.length === 0) {
			return;
		}

		console.log(`🪝 Running ${hookType} hooks...`);

		for (const hook of hooks) {
			try {
				// Skip commented lines
				if (hook.trim().startsWith("#")) {
					console.log(`   Skipping commented hook: ${hook}`);
					continue;
				}

				// Replace placeholders in the hook command
				let command = hook;
				if (context?.branchName) {
					command = command.replace(/\$\{branchName\}/g, context.branchName);
				}
				if (context?.worktreePath) {
					command = command.replace(/\$\{worktreePath\}/g, context.worktreePath);
				}

				console.log(`   Executing: ${command}`);
				
				// Execute the hook in the working directory with real-time output
				// Using native exec with inherited stdio to ensure proper streaming
				const { exec } = await import('node:child_process');
				await new Promise<void>((resolve, reject) => {
					const child = exec(command, {
						cwd: workingDirectory,
						env: { ...process.env, FORCE_COLOR: '1' }
					});
					
					// Pipe stdout and stderr to parent process streams
					if (child.stdout) child.stdout.pipe(process.stdout);
					if (child.stderr) child.stderr.pipe(process.stderr);
					
					child.on('close', (code) => {
						if (code === 0) {
							resolve();
						} else {
							reject(new Error(`Command failed with exit code ${code}`));
						}
					});
					
					child.on('error', reject);
				});
				
				console.log(`   ✓ Hook completed successfully`);
			} catch (error) {
				console.warn(`   ⚠️  Hook failed: ${error instanceof Error ? error.message : String(error)}`);
				// Continue with other hooks even if one fails
			}
		}
	} catch (error) {
		console.warn(`⚠️  Failed to execute hooks: ${error instanceof Error ? error.message : String(error)}`);
	}
}