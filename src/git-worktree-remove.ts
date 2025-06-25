#!/usr/bin/env tsx

import { join } from "node:path";
import { existsSync } from "node:fs";
import { $ } from "zx";
import { execSync } from "node:child_process";
import { getAllWorktrees, getCurrentWorktreePath, type Worktree, cleanBranchName, executeHooks } from "./git-worktree-utils.js";

$.verbose = false;
// Detect user's shell or use zsh as fallback
const userShell = process.env.SHELL || "/bin/zsh";
$.shell = userShell;

async function findWorktreeToRemove(
  branchName: string,
  worktrees: Worktree[]
): Promise<Worktree | null> {
  // Try to find worktree by branch name
  let targetWorktree = worktrees.find((wt) => {
    const cleanBranch = cleanBranchName(wt.branch);
    return cleanBranch === branchName;
  });

  // If not found by branch name, try by path (last part of path)
  if (!targetWorktree) {
    targetWorktree = worktrees.find((wt) => {
      const pathParts = wt.path.split("/");
      const lastPart = pathParts[pathParts.length - 1];
      return lastPart === branchName;
    });
  }

  return targetWorktree || null;
}

async function gwtremove(branchName?: string) {
  try {
    const worktrees = await getAllWorktrees();

    if (worktrees.length === 0) {
      console.error("No worktrees found.");
      process.exit(1);
    }

    let targetWorktree: Worktree | null = null;

    if (!branchName) {
      // No parameter given, remove current worktree
      const currentPath = await getCurrentWorktreePath();
      if (!currentPath) {
        console.error(
          "Error: Not in a git worktree. Please specify a branch to remove."
        );
        process.exit(1);
      }

      targetWorktree = worktrees.find((wt) => wt.path === currentPath) || null;
      if (!targetWorktree) {
        console.error("Error: Current directory is not a known worktree.");
        process.exit(1);
      }
    } else {
      // Parameter given, find the specified worktree
      targetWorktree = await findWorktreeToRemove(branchName, worktrees);
      if (!targetWorktree) {
        console.error(`Error: Worktree for '${branchName}' not found.`);
        console.log("\nAvailable worktrees:");
        for (const worktree of worktrees) {
          const cleanBranch = cleanBranchName(worktree.branch) || worktree.HEAD.slice(0, 7);
          console.log(`  ${cleanBranch} -> ${worktree.path}`);
        }
        process.exit(1);
      }
    }

    // Check if this is the main/bare repository
    if (targetWorktree.bare) {
      console.error("Error: Cannot remove the main (bare) repository.");
      process.exit(1);
    }

    const cleanBranch = cleanBranchName(targetWorktree.branch) || targetWorktree.HEAD.slice(0, 7);

    // Confirm removal
    console.log(`About to remove worktree:`);
    console.log(`  Path: ${targetWorktree.path}`);
    console.log(`  Branch: ${cleanBranch}`);

    // Check if we're currently in the worktree being removed
    const currentPath = await getCurrentWorktreePath();
    const willRemoveCurrent = currentPath === targetWorktree.path;

    if (willRemoveCurrent) {
      console.log(
        "\n⚠️  You are currently in this worktree. You will be moved to the project root after removal."
      );
    }

    // Ask for confirmation
    console.log("");
    const confirmation = await new Promise<string>((resolve) => {
      process.stdout.write(
        "Are you sure you want to remove this worktree? (y/N): "
      );
      process.stdin.setEncoding("utf8");
      process.stdin.once("data", (data) => {
        // Clean up stdin immediately
        process.stdin.pause();
        resolve(data.toString().trim());
      });
    });

    if (
      confirmation.toLowerCase() !== "y" &&
      confirmation.toLowerCase() !== "yes"
    ) {
      console.log("Removal cancelled.");
      process.exit(0);
    }

    // Find the main branch worktree to run commands from
    const mainBranches = ["main", "master", "dev", "develop"];
    let mainBranchWorktree = worktrees.find((wt) => {
      const branchName = cleanBranchName(wt.branch);
      return mainBranches.includes(branchName);
    });

    // If no main branch found, use any other worktree
    if (!mainBranchWorktree) {
      mainBranchWorktree = worktrees.find(
        (wt) => wt.path !== targetWorktree!.path
      );
    }

    if (!mainBranchWorktree) {
      throw new Error("No other worktrees found to execute git command from.");
    }

    const gitWorkingDir = mainBranchWorktree.path;

    // Determine the project root (parent directory of all worktrees)
    let projectRoot: string;
    const configPath = join(process.cwd(), "git-worktree-config.yaml");
    if (existsSync(configPath)) {
      projectRoot = process.cwd();
    } else {
      // Find project root by going up from any worktree path
      const pathParts = gitWorkingDir.split("/");
      projectRoot = pathParts.slice(0, -1).join("/");
    }

    // Change to main branch worktree to perform operations
    const originalCwd = process.cwd();
    process.chdir(gitWorkingDir);

    try {
      // Remove the worktree
      console.log("\nRemoving worktree...");
      await $`git worktree remove ${targetWorktree.path} --force`;

      console.log(`✓ Worktree removed: ${targetWorktree.path}`);

      // Also delete the branch if it's not a main branch
      if (!mainBranches.includes(cleanBranch)) {
        try {
          // Use execSync for more reliable execution
          execSync(`git branch -D "${cleanBranch}"`, { 
            cwd: process.cwd(),
            stdio: 'pipe',
            encoding: 'utf8'
          });
          console.log(`✓ Branch deleted: ${cleanBranch}`);
        } catch (error) {
          // Branch deletion failed, but worktree was removed
          console.log(
            `⚠️  Branch '${cleanBranch}' could not be deleted automatically`
          );
          console.log(
            `   (Error: ${
              error instanceof Error ? error.message : String(error)
            })`
          );
        }
      } else {
        console.log(`✓ Branch: ${cleanBranch} (preserved - main branch)`);
      }

      // Execute post-remove hooks
      await executeHooks("postRemove", projectRoot, {
        branchName: cleanBranch,
        worktreePath: targetWorktree.path
      });
    } finally {
      // If we removed the current worktree, go to project root
      // Otherwise, return to original directory
      if (willRemoveCurrent) {
        process.chdir(projectRoot);
        console.log(`✓ Moved to project root: ${projectRoot}`);
      } else {
        process.chdir(originalCwd);
      }
    }
  } catch (error) {
    console.error(
      "Error:",
      error instanceof Error ? error.message : String(error)
    );
    process.exit(1);
  }
}

const branchName = process.argv[2];
await gwtremove(branchName);
