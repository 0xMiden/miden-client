#!/usr/bin/env node
import { cpSync, mkdirSync, rmSync, existsSync, readdirSync } from "node:fs";
import { execSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const webClientDir = resolve(__dirname, "..");
const idxdbJsDir = resolve(webClientDir, "../idxdb-store/src/js");
const nodeStoreDir = resolve(webClientDir, "js/node-store");
const backupDir = resolve(webClientDir, ".idxdb-backup");

console.log("=== Building Node.js variant of web-client ===\n");

// 1. Backup original idxdb-store JS files
console.log("1. Backing up idxdb-store/src/js/ ...");
if (existsSync(backupDir)) rmSync(backupDir, { recursive: true });
mkdirSync(backupDir, { recursive: true });
cpSync(idxdbJsDir, backupDir, { recursive: true });

try {
  // 2. Copy node-store files over idxdb-store JS files
  console.log("2. Swapping in node-store files ...");
  const nodeStoreFiles = readdirSync(nodeStoreDir);
  for (const file of nodeStoreFiles) {
    cpSync(resolve(nodeStoreDir, file), resolve(idxdbJsDir, file));
  }
  console.log(
    `   Swapped ${nodeStoreFiles.length} files: ${nodeStoreFiles.join(", ")}`
  );

  // 3. Remove dist-node if it exists
  const distNodeDir = resolve(webClientDir, "dist-node");
  if (existsSync(distNodeDir)) {
    rmSync(distNodeDir, { recursive: true });
  }

  // 4. Build with node Rollup config
  console.log("3. Building dist-node/ ...");
  const env = { ...process.env };
  // Forward MIDEN_WEB_DEV if set
  execSync("npx rollup -c rollup.config.node.js", {
    cwd: webClientDir,
    stdio: "inherit",
    env,
  });
} finally {
  // 5. Restore original files (always, even on error)
  console.log("\n4. Restoring original idxdb-store/src/js/ ...");
  cpSync(backupDir, idxdbJsDir, { recursive: true });
  rmSync(backupDir, { recursive: true });
}

console.log("\n=== Node.js build complete → dist-node/ ===");
