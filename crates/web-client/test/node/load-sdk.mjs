/**
 * Shared SDK loader for Node.js tests.
 *
 * Centralizes the dist-node path resolution and provides a clear error
 * when the build artifacts are missing.
 */

import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { existsSync } from "node:fs";

const DIST_NODE_DIR = resolve(
  dirname(fileURLToPath(import.meta.url)),
  "../../dist-node"
);

/**
 * Dynamically import the Node.js SDK from the local dist-node build.
 *
 * @returns {Promise<object>} The SDK module namespace.
 * @throws {Error} If dist-node/node-entry.js does not exist.
 */
export async function loadSdk() {
  const entry = resolve(DIST_NODE_DIR, "node-entry.js");
  if (!existsSync(entry)) {
    throw new Error(
      `Node.js SDK not found at ${entry}.\n` +
        `Run "node scripts/build-node.mjs" from crates/web-client/ first.`
    );
  }
  return import(entry);
}
