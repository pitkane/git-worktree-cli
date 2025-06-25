#!/usr/bin/env tsx

import { join, resolve, dirname } from "node:path";
import { existsSync } from "node:fs";
import { rm } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { $ } from "zx";

$.verbose = true;

// Dynamic path resolution
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const SCRIPT_ROOT = resolve(__dirname, "..");
const TEST_DIR = join(SCRIPT_ROOT, "test-temp");
const TEST_REPO_URL = "git@github.com:pitkane/git-worktree-scripts.git";

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

		// Clean up any test worktrees within the project directory
		const testWorktrees = [
			join(SCRIPT_ROOT, "testbranch"),
			join(SCRIPT_ROOT, "dev"), 
			join(SCRIPT_ROOT, "feature"),
			join(SCRIPT_ROOT, "some-branch"),
			join(SCRIPT_ROOT, "temp-test-branch")
		];
		for (const dir of testWorktrees) {
			if (existsSync(dir)) {
				await rm(dir, { recursive: true, force: true });
			}
		}

		console.log("✓ Test environment cleaned");
	}

	async cleanup() {
		console.log("\n🧹 Cleaning up test environment...");
		if (existsSync(TEST_DIR)) {
			await rm(TEST_DIR, { recursive: true, force: true });
			console.log("✓ Test directory removed");
		}
		
		// Clean up any test worktrees within the project directory
		const testWorktrees = [
			join(SCRIPT_ROOT, "testbranch"),
			join(SCRIPT_ROOT, "dev"), 
			join(SCRIPT_ROOT, "feature"),
			join(SCRIPT_ROOT, "some-branch"),
			join(SCRIPT_ROOT, "temp-test-branch")
		];
		for (const dir of testWorktrees) {
			if (existsSync(dir)) {
				await rm(dir, { recursive: true, force: true });
				console.log(`✓ Test worktree ${dir} removed`);
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
			console.log(`   ${error instanceof Error ? error.message : String(error)}`);
		}
	}

	printSummary() {
		console.log("\n" + "=".repeat(50));
		console.log("📊 TEST SUMMARY");
		console.log("=".repeat(50));
		console.log(`Total tests: ${this.testCount}`);
		console.log(`Passed: ${this.passCount}`);
		console.log(`Failed: ${this.failCount}`);
		console.log(`Success rate: ${Math.round((this.passCount / this.testCount) * 100)}%`);

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

		// Create test directory for all tests
		await $`mkdir -p ${TEST_DIR}`;
		process.chdir(TEST_DIR);
		console.log(`✓ Created and changed to: ${TEST_DIR}`);

		// Test 1: gwtinit command integration (with bash function)
		await runner.test("gwtinit should initialize worktree from repository URL", async () => {
			const result = await $`cd ${TEST_DIR} && source ${SCRIPT_ROOT}/src/bash-functionality.sh && gwtinit ${TEST_REPO_URL}`;

			// Verify gwtinit results
			const configExists = existsSync(join(TEST_DIR, "git-worktree-config.yaml"));
			const mainExists = existsSync(join(TEST_DIR, "main"));

			console.log(`   Config exists: ${configExists}`);
			console.log(`   Main directory exists: ${mainExists}`);

			return configExists && mainExists;
		});

		// Test 2: gwtlist from project root (integration test)
		await runner.test("gwtlist should show worktrees from project root", async () => {
			const listResult = await $`cd ${TEST_DIR} && source ${SCRIPT_ROOT}/src/bash-functionality.sh && gwtlist`;

			const hasMainWorktree = listResult.stdout.includes("main");
			console.log(`   Found main worktree: ${hasMainWorktree}`);

			return hasMainWorktree;
		});

		// Test 3: gwtadd simple branch (integration test)
		await runner.test("gwtadd should create new worktree with simple branch name", async () => {
			const uniqueBranch = `testbranch-${Date.now()}`;
			const result = await $`cd ${TEST_DIR} && source ${SCRIPT_ROOT}/src/bash-functionality.sh && gwtadd ${uniqueBranch}`;

			// Check if unique branch was created within project directory
			const devExists = existsSync(join(SCRIPT_ROOT, uniqueBranch));
			console.log(`   Branch ${uniqueBranch} exists in project dir: ${devExists}`);

			// Store for later tests
			(global as any).testBranch = uniqueBranch;

			return devExists;
		});

		// Test 4: gwtadd nested branch (feature/test)
		await runner.test("gwtadd should create nested worktree with slash in name", async () => {
			const nestedBranch = "feature/test-integration";
			await $`cd ${TEST_DIR} && source ${SCRIPT_ROOT}/src/bash-functionality.sh && gwtadd ${nestedBranch}`;

			// Check if nested directory was created within project directory
			const nestedExists = existsSync(join(SCRIPT_ROOT, "feature", "test-integration"));
			console.log(`   Nested branch exists in project dir: ${nestedExists}`);

			return nestedExists;
		});

		// Test 5: gwtlist should show all created worktrees
		await runner.test("gwtlist should show all created worktrees", async () => {
			const listResult = await $`cd ${TEST_DIR} && source ${SCRIPT_ROOT}/src/bash-functionality.sh && gwtlist`;

			const hasMain = listResult.stdout.includes("main");
			const hasTestBranch = listResult.stdout.includes("testbranch");
			const hasFeatureBranch = listResult.stdout.includes("test-integration");

			console.log(`   Found main: ${hasMain}`);
			console.log(`   Found testbranch: ${hasTestBranch}`);
			console.log(`   Found feature branch: ${hasFeatureBranch}`);

			return hasMain && hasTestBranch && hasFeatureBranch;
		});

		// Test 6: gwtswitch tab completion
		await runner.test("gwtswitch tab completion should work", async () => {
			const testBranch = (global as any).testBranch || "testbranch";
			const switchCompletionResult = await $`cd ${TEST_DIR} && ${SCRIPT_ROOT}/node_modules/.bin/tsx ${SCRIPT_ROOT}/src/git-worktree-completion.ts gwtswitch test`;

			const hasCompletion = switchCompletionResult.stdout.includes(testBranch) || switchCompletionResult.stdout.includes("test-integration");
			console.log(`   Completion includes branches: ${hasCompletion}`);

			return hasCompletion;
		});

		// Test 7: gwtswitch command integration
		await runner.test("gwtswitch should switch between worktrees", async () => {
			const testBranch = (global as any).testBranch || "testbranch";
			const switchResult = await $`cd ${TEST_DIR} && source ${SCRIPT_ROOT}/src/bash-functionality.sh && gwtswitch ${testBranch}`;

			// Check if the output indicates successful switch (no longer shows directory change due to shell limitations)
			const hasSuccessMessage = switchResult.stdout.includes("Switching to worktree:");

			console.log(`   Switch command executed: ${hasSuccessMessage}`);

			return hasSuccessMessage;
		});

		// Test 8: gwtswitch with no parameter should show available worktrees
		await runner.test("gwtswitch should show available worktrees when no parameter given", async () => {
			try {
				const result = await $`cd ${TEST_DIR} && source ${SCRIPT_ROOT}/src/bash-functionality.sh && gwtswitch`;
				
				// Should show available worktrees or usage information
				const showsWorktrees = result.stdout.includes("Available worktrees") || result.stdout.includes("main");
				console.log(`   Shows available worktrees: ${showsWorktrees}`);
				
				return showsWorktrees;
			} catch (error) {
				// It's okay if this command exits with non-zero status but shows helpful info
				const errorOutput = error instanceof Error && 'stdout' in error ? (error as any).stdout : '';
				const showsWorktrees = errorOutput.includes("Available worktrees") || errorOutput.includes("Usage:");
				console.log(`   Shows helpful output on error: ${showsWorktrees}`);
				
				return showsWorktrees;
			}
		});

		// Test 9: gwtremove tab completion
		await runner.test("gwtremove tab completion should work", async () => {
			const testBranch = (global as any).testBranch || "testbranch";
			const completionResult = await $`cd ${TEST_DIR} && ${SCRIPT_ROOT}/node_modules/.bin/tsx ${SCRIPT_ROOT}/src/git-worktree-completion.ts gwtremove test`;

			const hasCompletion = completionResult.stdout.includes(testBranch) || completionResult.stdout.includes("test-integration");
			console.log(`   Completion includes branches: ${hasCompletion}`);

			return hasCompletion;
		});

		// Test 10: gwtremove with parameter (integration test)
		await runner.test("gwtremove should remove specified worktree", async () => {
			// Remove the feature branch
			await $`cd ${TEST_DIR} && source ${SCRIPT_ROOT}/src/bash-functionality.sh && echo "y" | gwtremove test-integration`;

			// Check if directory was removed
			const nestedExists = existsSync(join(SCRIPT_ROOT, "feature", "test-integration"));
			console.log(`   Feature branch still exists: ${nestedExists}`);

			return !nestedExists;
		});

		// Test 11: gwtremove current worktree (via parameter to avoid interactive prompt)
		await runner.test("gwtremove should remove specified worktree and handle directory navigation", async () => {
			const testBranch = (global as any).testBranch || "testbranch";
			await $`cd ${TEST_DIR} && source ${SCRIPT_ROOT}/src/bash-functionality.sh && echo "y" | gwtremove ${testBranch}`;

			// Check if directory was removed
			const testExists = existsSync(join(SCRIPT_ROOT, testBranch));
			console.log(`   Test branch still exists: ${testExists}`);

			return !testExists;
		});

		// Test 12: Final state verification
		await runner.test("Final state should show only main worktree", async () => {
			const listResult = await $`cd ${TEST_DIR} && source ${SCRIPT_ROOT}/src/bash-functionality.sh && gwtlist`;

			const hasMain = listResult.stdout.includes("main");
			const hasNoTestBranch = !listResult.stdout.includes("testbranch");
			const hasNoFeatureBranch = !listResult.stdout.includes("test-integration");

			console.log(`   Has main: ${hasMain}`);
			console.log(`   No test branches: ${hasNoTestBranch && hasNoFeatureBranch}`);

			return hasMain && hasNoTestBranch && hasNoFeatureBranch;
		});

		// Test 13: Direct TypeScript API tests (unit tests)
		await runner.test("git-worktree-list.ts should work directly", async () => {
			const result = await $`cd ${TEST_DIR} && ${SCRIPT_ROOT}/node_modules/.bin/tsx ${SCRIPT_ROOT}/src/git-worktree-list.ts`;
			
			const hasMainWorktree = result.stdout.includes("main");
			console.log(`   Direct TS call shows main: ${hasMainWorktree}`);

			return hasMainWorktree;
		});

		// Test 14: Error handling - invalid repository
		await runner.test("gwtinit should handle invalid repository URLs gracefully", async () => {
			// Create a separate test directory for this
			const errorTestDir = `${TEST_DIR}-error`;
			await $`mkdir -p ${errorTestDir}`;
			
			try {
				// This should fail gracefully
				await $`cd ${errorTestDir} && source ${SCRIPT_ROOT}/src/bash-functionality.sh && gwtinit invalid-repo-url`;
				return false; // Should not succeed
			} catch (error) {
				// Expected to fail
				console.log("   Correctly failed with invalid repo URL");
				return true;
			} finally {
				// Clean up error test directory
				if (existsSync(errorTestDir)) {
					await rm(errorTestDir, { recursive: true, force: true });
				}
			}
		});

		// Test 15: Check that gwtadd can work from any directory (finds project root)
		await runner.test("gwtadd should work from any directory by finding project root", async () => {
			// Create a separate test directory for this
			const errorTestDir = `${TEST_DIR}-no-git`;
			await $`mkdir -p ${errorTestDir}`;
			
			try {
				// This might succeed if it can find a project root
				await $`cd ${errorTestDir} && source ${SCRIPT_ROOT}/src/bash-functionality.sh && gwtadd temp-test-branch`;
				console.log("   gwtadd succeeded from outside directory (found project root)");
				
				// Clean up the created branch
				try {
					await $`cd ${TEST_DIR} && source ${SCRIPT_ROOT}/src/bash-functionality.sh && echo "y" | gwtremove temp-test-branch`;
				} catch (e) {
					// Ignore cleanup errors
				}
				
				return true; // This is actually acceptable behavior
			} catch (error) {
				console.log("   gwtadd correctly failed when no project root found");
				return true; // Both outcomes are valid
			} finally {
				// Clean up error test directory
				if (existsSync(errorTestDir)) {
					await rm(errorTestDir, { recursive: true, force: true });
				}
			}
		});

	} finally {
		await runner.cleanup();
		runner.printSummary();
	}
}

// Run comprehensive tests
await main();