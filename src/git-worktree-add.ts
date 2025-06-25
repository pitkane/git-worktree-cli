#!/usr/bin/env tsx

import { join } from "node:path";
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
            const match = remoteBranches.match(/origin\/([^\s\-]+)/);
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

async function branchExists(
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
    const branchName = folderName;

    // Determine git root and target path
    let gitRoot: string | null = null;
    let targetPath: string;
    let gitWorkingDir: string;

    // Check if we're in a git repository
    gitRoot = await findGitRoot();

    if (gitRoot) {
      // We're in a git repository, find the project root (where git-worktree-config.yaml is)
      let projectRoot = gitRoot;
      let currentPath = gitRoot;

      // Walk up to find the project root with git-worktree-config.yaml
      while (currentPath !== "/" && currentPath !== ".") {
        if (existsSync(join(currentPath, "git-worktree-config.yaml"))) {
          projectRoot = currentPath;
          break;
        }
        currentPath = join(currentPath, "..");
      }

      // Create worktree relative to project root
      targetPath = join(projectRoot, folderName);
      gitWorkingDir = gitRoot;
    } else {
      // We're in project root, look for existing worktree to use as git repository
      const configPath = join(process.cwd(), "git-worktree-config.yaml");
      if (!existsSync(configPath)) {
        throw new Error(
          "Not in a git repository or project root with git-worktree-config.yaml"
        );
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
        throw new Error(
          "No existing worktrees found. Create one first using gwtinit."
        );
      }

      gitWorkingDir = foundWorktree;
      targetPath = join(process.cwd(), folderName);
    }

    const mainBranch = await getMainBranch(gitWorkingDir);
    const branchInfo = await branchExists(branchName, gitWorkingDir);

    if (branchInfo.local) {
      console.log(
        `Branch '${branchName}' exists locally, checking out existing branch...`
      );
      await $`cd ${gitWorkingDir} && git worktree add ${targetPath} ${branchName}`;
    } else if (branchInfo.remote) {
      console.log(
        `Branch '${branchName}' exists remotely, checking out remote branch...`
      );
      await $`cd ${gitWorkingDir} && git worktree add ${targetPath} -b ${branchName} origin/${branchName}`;
    } else {
      console.log(
        `Creating new branch '${branchName}' from 'origin/${mainBranch}'...`
      );
      await $`cd ${gitWorkingDir} && git worktree add ${targetPath} -b ${branchName} origin/${mainBranch}`;
    }

    console.log(`✓ Worktree created at: ${targetPath}`);
    console.log(`✓ Branch: ${branchName}`);
  } catch (error) {
    console.error(
      "Error:",
      error instanceof Error ? error.message : String(error)
    );
    process.exit(1);
  }
}

const folderName = process.argv[2];
await gwtadd(folderName);
