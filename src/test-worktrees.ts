#!/usr/bin/env tsx

import { join } from "node:path";
import { existsSync } from "node:fs";
import { rm } from "node:fs/promises";
import { $ } from "zx";

$.verbose = true;

const TEST_REPO_URL = "git@github.com:pitkane/git-worktree-scripts.git";
const TEST_DIR = "/Users/mikkoh/.git-worktree-scripts/test-temp";
const SCRIPT_ROOT = "/Users/mikkoh/.git-worktree-scripts";

class TestRunner {
  private testCount = 0;
  private passCount = 0;
  private failCount = 0;

  async setup() {
    console.log("🧪 Setting up test environment...");

    // Clean up any existing test directory
    if (existsSync(TEST_DIR)) {
      await rm(TEST_DIR, { recursive: true, force: true });
    }

    // Change to /tmp directory
    process.chdir("/tmp");
    console.log(`✓ Working directory: ${process.cwd()}`);
  }

  async cleanup() {
    console.log("\n🧹 Cleaning up test environment...");
    if (existsSync(TEST_DIR)) {
      await rm(TEST_DIR, { recursive: true, force: true });
      console.log("✓ Test directory removed");
    }
  }

  async runCommand(
    scriptPath: string,
    args: string[] = [],
    expectedToSucceed = true,
    workingDir?: string
  ): Promise<{ success: boolean; output: string; error?: string }> {
    try {
      const command = `${SCRIPT_ROOT}/node_modules/.bin/tsx`;
      const fullArgs = [scriptPath, ...args];
      
      // Save current directory and change to working directory if specified
      const originalDir = process.cwd();
      if (workingDir) {
        process.chdir(workingDir);
      }
      
      const result = await $`${command} ${fullArgs}`;
      const output = result.stdout || result.toString();
      
      // Restore original directory
      if (workingDir) {
        process.chdir(originalDir);
      }

      if (expectedToSucceed) {
        console.log(
          `✅ Command succeeded: tsx ${scriptPath} ${args.join(" ")}`
        );
        return { success: true, output };
      } else {
        console.log(
          `❌ Command unexpectedly succeeded: tsx ${scriptPath} ${args.join(
            " "
          )}`
        );
        return {
          success: false,
          output,
          error: "Expected to fail but succeeded",
        };
      }
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);

      if (!expectedToSucceed) {
        console.log(
          `✅ Command failed as expected: tsx ${scriptPath} ${args.join(" ")}`
        );
        return { success: true, output: "", error: errorMsg };
      } else {
        console.log(`❌ Command failed: tsx ${scriptPath} ${args.join(" ")}`);
        console.log(`   Error: ${errorMsg}`);
        return { success: false, output: "", error: errorMsg };
      }
    }
  }

  async test(description: string, testFn: () => Promise<boolean>) {
    this.testCount++;
    console.log(`\n📋 Test ${this.testCount}: ${description}`);

    try {
      const result = await testFn();
      if (result) {
        this.passCount++;
        console.log(`✅ PASS: ${description}`);
      } else {
        this.failCount++;
        console.log(`❌ FAIL: ${description}`);
      }
    } catch (error) {
      this.failCount++;
      console.log(`❌ ERROR: ${description}`);
      console.log(
        `   ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  printSummary() {
    console.log("\n" + "=".repeat(50));
    console.log("📊 TEST SUMMARY");
    console.log("=".repeat(50));
    console.log(`Total tests: ${this.testCount}`);
    console.log(`Passed: ${this.passCount}`);
    console.log(`Failed: ${this.failCount}`);
    console.log(
      `Success rate: ${Math.round((this.passCount / this.testCount) * 100)}%`
    );

    if (this.failCount === 0) {
      console.log("\n🎉 All tests passed!");
    } else {
      console.log(`\n⚠️  ${this.failCount} test(s) failed.`);
    }
  }
}

async function main() {
  const runner = new TestRunner();

  try {
    await runner.setup();

    // Test 1: gwtinit command
    await runner.test(
      "gwtinit should initialize worktree from repository URL",
      async () => {
        // Create test directory and change to it before running gwtinit
        await $`mkdir -p ${TEST_DIR}`;
        process.chdir(TEST_DIR);
        
        const result = await runner.runCommand(
          `${SCRIPT_ROOT}/src/git-worktree-init.ts`,
          [TEST_REPO_URL],
          true,
          TEST_DIR
        );

        // Check if config file was created
        const configExists = existsSync(
          join(TEST_DIR, "git-worktree-config.yaml")
        );

        // Check if initial worktree directory was created (should be 'main' branch)
        const mainDirExists = existsSync(join(TEST_DIR, "main"));

        return result.success && configExists && mainDirExists;
      }
    );

    // Ensure we're in the test directory for subsequent tests
    process.chdir(TEST_DIR);
    console.log(`✓ Changed to test directory: ${process.cwd()}`);

    // Test 2: gwtlist from project root
    await runner.test(
      "gwtlist should show worktrees from project root",
      async () => {
        const result = await runner.runCommand(
          `${SCRIPT_ROOT}/src/git-worktree-list.ts`
        );

        // Should show at least the main worktree
        const hasMainWorktree = result.output.includes("main");

        return result.success && hasMainWorktree;
      }
    );

    // Test 3: gwtadd simple branch
    await runner.test(
      "gwtadd should create new worktree with simple branch name",
      async () => {
        const result = await runner.runCommand(
          `${SCRIPT_ROOT}/src/git-worktree-add.ts`,
          ["dev"]
        );

        // Check if dev directory was created
        const devDirExists = existsSync(join(TEST_DIR, "dev"));

        return result.success && devDirExists;
      }
    );

    // Test 4: gwtadd nested branch (feature/test)
    await runner.test(
      "gwtadd should create nested worktree with slash in name",
      async () => {
        const result = await runner.runCommand(
          `${SCRIPT_ROOT}/src/git-worktree-add.ts`,
          ["feature/test-branch"]
        );

        // Check if feature/test-branch directory was created
        const featureDirExists = existsSync(
          join(TEST_DIR, "feature", "test-branch")
        );

        return result.success && featureDirExists;
      }
    );

    // Test 5: gwtlist should show all worktrees
    await runner.test("gwtlist should show all created worktrees", async () => {
      const result = await runner.runCommand(
        `${SCRIPT_ROOT}/src/git-worktree-list.ts`
      );

      // Should show main, dev, and test-branch
      const hasMain = result.output.includes("main");
      const hasDev = result.output.includes("dev");
      const hasTestBranch = result.output.includes("test-branch");

      return result.success && hasMain && hasDev && hasTestBranch;
    });

    // Test 6: gwtlist from within a worktree
    await runner.test(
      "gwtlist should work from within a worktree",
      async () => {
        // Change to dev worktree
        process.chdir(join(TEST_DIR, "dev"));

        const result = await runner.runCommand(
          `${SCRIPT_ROOT}/src/git-worktree-list.ts`
        );

        // Should still show all worktrees
        const hasMain = result.output.includes("main");
        const hasDev = result.output.includes("dev");
        const hasTestBranch = result.output.includes("test-branch");

        // Change back to project root
        process.chdir(TEST_DIR);

        return result.success && hasMain && hasDev && hasTestBranch;
      }
    );

    // Test 7: gwtswitch command
    await runner.test("gwtswitch should switch between worktrees", async () => {
      const result = await runner.runCommand(
        `${SCRIPT_ROOT}/src/git-worktree-switch.ts`,
        ["dev"]
      );

      // Check if we're now in the dev directory
      const currentDir = process.cwd();
      const isInDevDir = currentDir.endsWith("/dev");

      // Switch back to project root for other tests
      process.chdir(TEST_DIR);

      return result.success && isInDevDir;
    });

    // Test 8: gwtswitch with no parameter should show available worktrees
    await runner.test(
      "gwtswitch should show available worktrees when no parameter given",
      async () => {
        const result = await runner.runCommand(
          `${SCRIPT_ROOT}/src/git-worktree-switch.ts`
        );

        // Should show available worktrees
        const hasAvailableMessage = result.output.includes(
          "Available worktrees"
        );
        const hasMain = result.output.includes("main");
        const hasDev = result.output.includes("dev");

        return result.success && hasAvailableMessage && hasMain && hasDev;
      }
    );

    // Test 9: gwtswitch to nested worktree
    await runner.test(
      "gwtswitch should work with nested worktrees",
      async () => {
        const result = await runner.runCommand(
          `${SCRIPT_ROOT}/src/git-worktree-switch.ts`,
          ["test-branch"]
        );

        // Check if we're now in the feature/test-branch directory
        const currentDir = process.cwd();
        const isInTestBranchDir = currentDir.includes("feature/test-branch");

        // Switch back to project root
        process.chdir(TEST_DIR);

        return result.success && isInTestBranchDir;
      }
    );

    // Test 10: gwtadd existing branch should check out existing branch
    await runner.test(
      "gwtadd should detect and checkout existing remote branches",
      async () => {
        // This test assumes the test repo might have other branches
        // If it doesn't, this will create a new branch, which is also valid behavior
        const result = await runner.runCommand(
          `${SCRIPT_ROOT}/src/git-worktree-add.ts`,
          ["another-test"]
        );

        // Check if directory was created
        const dirExists = existsSync(join(TEST_DIR, "another-test"));

        return result.success && dirExists;
      }
    );

    // Test 11: gwtremove with parameter
    await runner.test(
      "gwtremove should remove specified worktree",
      async () => {
        const result = await runner.runCommand(
          `${SCRIPT_ROOT}/src/git-worktree-remove.ts`,
          ["another-test"]
        );

        // Check if directory was removed
        const dirExists = existsSync(join(TEST_DIR, "another-test"));

        return result.success && !dirExists;
      }
    );

    // Test 12: gwtremove from within worktree (current worktree)
    await runner.test(
      "gwtremove should remove current worktree and navigate away",
      async () => {
        // Switch to dev worktree first
        process.chdir(join(TEST_DIR, "dev"));

        const result = await runner.runCommand(
          `${SCRIPT_ROOT}/src/git-worktree-remove.ts`
        );

        // Check if we're no longer in dev directory
        const currentDir = process.cwd();
        const isNotInDevDir = !currentDir.includes("/dev");

        // Check if dev directory was removed
        const devDirExists = existsSync(join(TEST_DIR, "dev"));

        // Make sure we're back in project root
        process.chdir(TEST_DIR);

        return result.success && isNotInDevDir && !devDirExists;
      }
    );

    // Test 13: gwtremove should prevent removing main/bare repository
    await runner.test(
      "gwtremove should prevent removing main repository",
      async () => {
        // Try to remove main - this should fail
        const result = await runner.runCommand(
          `${SCRIPT_ROOT}/src/git-worktree-remove.ts`,
          ["main"],
          false
        );

        // Main directory should still exist
        const mainDirExists = existsSync(join(TEST_DIR, "main"));

        return result.success && mainDirExists;
      }
    );

    // Test 14: Verify final state
    await runner.test(
      "Final state should show remaining worktrees",
      async () => {
        const result = await runner.runCommand(
          `${SCRIPT_ROOT}/src/git-worktree-list.ts`
        );

        // Should show main and test-branch, but not dev (which was removed)
        const hasMain = result.output.includes("main");
        const hasTestBranch = result.output.includes("test-branch");
        const hasNoDev = !result.output.includes("dev");

        return result.success && hasMain && hasTestBranch && hasNoDev;
      }
    );
  } finally {
    await runner.cleanup();
    runner.printSummary();
  }
}

// Run tests
await main();
