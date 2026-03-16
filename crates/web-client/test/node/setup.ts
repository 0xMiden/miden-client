import { test as base } from "@playwright/test";
import { createRequire } from "module";
import path from "path";
import fs from "fs";
import os from "os";

const require = createRequire(import.meta.url);

// Resolve the napi binary. Check common locations in order:
// 1. MIDEN_MODULE_PATH env var
// 2. Repo build output for the current platform
function resolveNativeModule(): string {
  if (process.env.MIDEN_MODULE_PATH) {
    return process.env.MIDEN_MODULE_PATH;
  }

  const repoRoot = path.resolve(import.meta.dirname, "..", "..", "..", "..");
  const arch = os.arch() === "arm64" ? "aarch64" : os.arch();
  const platform =
    os.platform() === "darwin" ? "apple-darwin" : `unknown-linux-gnu`;
  const target = `${arch}-${platform}`;
  const ext = os.platform() === "darwin" ? "dylib" : "so";

  const candidates = [
    path.join(
      repoRoot,
      "target",
      target,
      "release",
      `libmiden_client_web.${ext}`
    ),
    path.join(repoRoot, "target", "release", `libmiden_client_web.${ext}`),
  ];

  for (const p of candidates) {
    if (fs.existsSync(p)) {
      // Node.js requires native modules to have a .node extension.
      // Copy the dylib/so with the .node extension if needed.
      const nodeFile = path.join(path.dirname(p), "miden_client_web.node");
      if (
        !fs.existsSync(nodeFile) ||
        fs.statSync(p).mtimeMs > fs.statSync(nodeFile).mtimeMs
      ) {
        fs.copyFileSync(p, nodeFile);
      }
      return nodeFile;
    }
  }

  throw new Error(
    `Could not find miden napi module. Build it with:\n` +
      `  cargo build -p miden-client-web --no-default-features --features nodejs,testing --release --target ${target}\n` +
      `Or set MIDEN_MODULE_PATH to the .node path.\n` +
      `Searched: ${candidates.join(", ")}`
  );
}

export const sdk = require(resolveNativeModule());

let testCounter = 0;

/**
 * Creates a fresh mock client backed by a temporary SQLite database.
 * Each call gets its own isolated store + keystore directory.
 */
export async function createMockClient() {
  const tmpDir = path.join(
    os.tmpdir(),
    `miden-test-${process.pid}-${++testCounter}`
  );
  fs.mkdirSync(path.join(tmpDir, "keystore"), { recursive: true });

  const client = new sdk.WebClient();
  await client.createMockClient(
    path.join(tmpDir, "store.db"),
    path.join(tmpDir, "keystore"),
    null,
    null,
    null
  );
  return { client, tmpDir };
}

/**
 * Custom Playwright test fixture that provides the SDK and a mock client factory.
 * No browser page is needed — tests run directly in Node.js.
 */
export const test = base.extend<{
  mockClient: { client: any; tmpDir: string };
}>({
  mockClient: async ({}, use) => {
    const ctx = await createMockClient();
    await use(ctx);
  },
});

export { expect } from "@playwright/test";
