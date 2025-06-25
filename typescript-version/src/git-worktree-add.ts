#!/usr/bin/env tsx

import { join } from "node:path";
import { existsSync } from "node:fs";
import { readdir } from "node:fs/promises";
import { $ } from "zx";
import { findGitRoot, getMainBranch, branchExists, executeHooks } from "./git-worktree-utils.js";

$.verbose = false;

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
      await $({ stdio: 'inherit' })`cd ${gitWorkingDir} && git worktree add ${targetPath} -b ${branchName} origin/${branchName}`;
    } else {
      console.log(
        `Creating new branch '${branchName}' from 'origin/${mainBranch}'...`
      );
      await $({ stdio: 'inherit' })`cd ${gitWorkingDir} && git worktree add ${targetPath} -b ${branchName} origin/${mainBranch}`;
    }

    console.log(`✓ Worktree created at: ${targetPath}`);
    console.log(`✓ Branch: ${branchName}`);

    // Execute post-add hooks
    await executeHooks("postAdd", targetPath, {
      branchName: branchName,
      worktreePath: targetPath
    });
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
