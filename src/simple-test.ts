#!/usr/bin/env tsx

import { join } from "node:path";
import { existsSync } from "node:fs";
import { rm } from "node:fs/promises";
import { $ } from "zx";

$.verbose = true;

const TEST_REPO_URL = "git@github.com:pitkane/git-worktree-scripts.git";
const TEST_DIR = "/Users/mikkoh/.git-worktree-scripts/test-temp";
const SCRIPT_ROOT = "/Users/mikkoh/.git-worktree-scripts";

async function cleanup() {
  console.log("🧹 Cleaning up...");
  if (existsSync(TEST_DIR)) {
    await rm(TEST_DIR, { recursive: true, force: true });
    console.log("✓ Test directory removed");
  }
  
  // Also clean up any worktrees that might have been created in home directory
  const homeTestBranch = "/Users/mikkoh/testbranch";
  if (existsSync(homeTestBranch)) {
    await rm(homeTestBranch, { recursive: true, force: true });
    console.log("✓ Home testbranch directory removed");
  }
}

async function runTest() {
  try {
    // Clean up and setup
    await cleanup();
    
    console.log("🧪 Running simple integration test...");
    
    // Create test directory
    await $`mkdir -p ${TEST_DIR}`;
    process.chdir(TEST_DIR);
    console.log(`✓ Created and changed to: ${TEST_DIR}`);
    
    // Test 1: gwtinit
    console.log("\n📋 Testing gwtinit...");
    await $`cd ${TEST_DIR} && source ${SCRIPT_ROOT}/src/bash-functionality.sh && gwtinit ${TEST_REPO_URL}`;
    
    // Verify gwtinit results
    const configExists = existsSync(join(TEST_DIR, "git-worktree-config.yaml"));
    const mainExists = existsSync(join(TEST_DIR, "main"));
    
    console.log(`   Checking: ${join(TEST_DIR, "git-worktree-config.yaml")}`);
    console.log(`   Checking: ${join(TEST_DIR, "main")}`);
    console.log(`   Config exists: ${configExists}`);
    console.log(`   Main directory exists: ${mainExists}`);
    
    // List what's actually in the test directory
    try {
      const files = await $`ls -la ${TEST_DIR}`;
      console.log(`   Directory contents:\n${files.stdout}`);
    } catch (error) {
      console.log(`   Could not list directory: ${error}`);
    }
    
    if (configExists && mainExists) {
      console.log("✅ gwtinit test passed");
    } else {
      console.log("❌ gwtinit test failed");
      return;
    }
    
    // Test 2: gwtlist from project root
    console.log("\n📋 Testing gwtlist from project root...");
    const listResult = await $`cd ${TEST_DIR} && source ${SCRIPT_ROOT}/src/bash-functionality.sh && gwtlist`;
    
    if (listResult.stdout.includes("main")) {
      console.log("✅ gwtlist test passed");
    } else {
      console.log("❌ gwtlist test failed");
      console.log("   Output:", listResult.stdout);
      return;
    }
    
    // Test 3: gwtadd
    console.log("\n📋 Testing gwtadd...");
    const uniqueBranch = `testbranch-${Date.now()}`;
    await $`cd ${TEST_DIR} && source ${SCRIPT_ROOT}/src/bash-functionality.sh && gwtadd ${uniqueBranch}`;
    
    // Check if unique branch was created anywhere (could be in user home or test dir)
    const devExists1 = existsSync(join(TEST_DIR, uniqueBranch));
    const devExists2 = existsSync(`/Users/mikkoh/${uniqueBranch}`);
    const devExists = devExists1 || devExists2;
    
    if (devExists) {
      console.log("✅ gwtadd test passed");
    } else {
      console.log("❌ gwtadd test failed");
      console.log(`   ${uniqueBranch} directory exists in test dir: ${devExists1}`);
      console.log(`   ${uniqueBranch} directory exists in home: ${devExists2}`);
      return;
    }
    
    // Test 4: gwtlist again (should show both worktrees)
    console.log("\n📋 Testing gwtlist after gwtadd...");
    const listResult2 = await $`cd ${TEST_DIR} && source ${SCRIPT_ROOT}/src/bash-functionality.sh && gwtlist`;
    
    if (listResult2.stdout.includes("main") && listResult2.stdout.includes(uniqueBranch.split('-')[0])) {
      console.log("✅ gwtlist after gwtadd test passed");
    } else {
      console.log("❌ gwtlist after gwtadd test failed");
      console.log("   Output:", listResult2.stdout);
      return;
    }
    
    // Test 5: gwtswitch
    console.log("\n📋 Testing gwtswitch...");
    const switchResult = await $`cd ${TEST_DIR} && source ${SCRIPT_ROOT}/src/bash-functionality.sh && gwtswitch ${uniqueBranch}`;
    
    // Check if the output indicates successful switch
    if (switchResult.stdout.includes("✓ Now in:") && switchResult.stdout.includes(uniqueBranch)) {
      console.log("✅ gwtswitch test passed");
    } else {
      console.log("❌ gwtswitch test failed");
      console.log("   Output:", switchResult.stdout);
      return;
    }
    
    // Switch back to project root
    process.chdir(TEST_DIR);
    
    // Test 6: gwtremove
    console.log("\n📋 Testing gwtremove...");
    await $`cd ${TEST_DIR} && source ${SCRIPT_ROOT}/src/bash-functionality.sh && gwtremove ${uniqueBranch}`;
    
    const devExistsAfterRemove1 = existsSync(join(TEST_DIR, uniqueBranch));
    const devExistsAfterRemove2 = existsSync(`/Users/mikkoh/${uniqueBranch}`);
    const devExistsAfterRemove = devExistsAfterRemove1 || devExistsAfterRemove2;
    
    if (!devExistsAfterRemove) {
      console.log("✅ gwtremove test passed");
    } else {
      console.log("❌ gwtremove test failed");
      console.log(`   ${uniqueBranch} directory exists after remove in test dir: ${devExistsAfterRemove1}`);
      console.log(`   ${uniqueBranch} directory exists after remove in home: ${devExistsAfterRemove2}`);
      return;
    }
    
    console.log("\n🎉 All tests passed!");
    
  } catch (error) {
    console.error("❌ Test failed with error:", error);
  } finally {
    await cleanup();
  }
}

await runTest();