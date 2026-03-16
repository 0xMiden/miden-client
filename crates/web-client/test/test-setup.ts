/**
 * Platform-agnostic test setup for both browser (Playwright) and Node.js (napi).
 *
 * Provides `client` and `sdk` fixtures that abstract the platform difference:
 * - Browser: SDK operations run inside a browser page via page.evaluate()
 * - Node.js: SDK operations run directly in the test process
 *
 * Tests import { test, expect } from "./test-setup" and use `client` and `sdk`
 * without knowing which platform they're on.
 */
// @ts-nocheck
import { test as base, expect } from "@playwright/test";
import type { TestInfo } from "@playwright/test";
import { createRequire } from "module";
import path from "path";
import fs from "fs";
import os from "os";
import { getRpcUrl, getProverUrl, RUN_ID } from "./playwright.global.setup";

const require = createRequire(import.meta.url);

function generateStoreName(testInfo: TestInfo): string {
  return `test_${RUN_ID}_${testInfo.testId}`;
}

// ── Node.js setup ─────────────────────────────────────────────────────

let _nodeSdk: any = null;

function loadNodeSdk(): any {
  if (_nodeSdk) return _nodeSdk;

  const repoRoot = path.resolve(import.meta.dirname, "..", "..", "..");
  const arch = os.arch() === "arm64" ? "aarch64" : os.arch();
  const platform =
    os.platform() === "darwin" ? "apple-darwin" : "unknown-linux-gnu";
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
      const nodeFile = path.join(path.dirname(p), "miden_client_web.node");
      if (
        !fs.existsSync(nodeFile) ||
        fs.statSync(p).mtimeMs > fs.statSync(nodeFile).mtimeMs
      ) {
        fs.copyFileSync(p, nodeFile);
      }
      _nodeSdk = require(nodeFile);
      return _nodeSdk;
    }
  }

  throw new Error(
    `napi module not found. Build with:\n` +
      `  cargo build -p miden-client-web --no-default-features --features nodejs,testing --release --target ${target}`
  );
}

let _nodeTestCounter = 0;

async function createNodeMockClient(): Promise<{ client: any; sdk: any }> {
  const rawSdk = loadNodeSdk();
  const tmpDir = path.join(
    os.tmpdir(),
    `miden-test-${process.pid}-${++_nodeTestCounter}`
  );
  fs.mkdirSync(path.join(tmpDir, "keystore"), { recursive: true });

  const rawClient = new rawSdk.WebClient();
  await rawClient.createMockClient(
    path.join(tmpDir, "store.db"),
    path.join(tmpDir, "keystore"),
    null,
    null,
    null
  );

  // Wrap the client to normalize napi differences
  const client = wrapNodeClient(rawClient, rawSdk);
  const sdk = createNodeSdkWrapper(rawSdk);

  return { client, sdk };
}

async function createNodeIntegrationClient(
  rpcUrl: string,
  storeName: string
): Promise<{ client: any; sdk: any }> {
  const rawSdk = loadNodeSdk();
  const tmpDir = path.join(
    os.tmpdir(),
    `miden-test-${process.pid}-${++_nodeTestCounter}`
  );
  fs.mkdirSync(path.join(tmpDir, "keystore"), { recursive: true });

  const rawClient = new rawSdk.WebClient();
  await rawClient.createClient(
    rpcUrl,
    null,
    null,
    path.join(tmpDir, `${storeName}.db`),
    path.join(tmpDir, "keystore"),
    false
  );

  const client = wrapNodeClient(rawClient, rawSdk);
  const sdk = createNodeSdkWrapper(rawSdk);

  return { client, sdk };
}

/**
 * Wraps a napi WebClient to normalize differences with the browser SDK:
 * - syncState() → syncStateImpl()
 * - null → undefined for Option<T> returns
 */
function wrapNodeClient(rawClient: any, rawSdk: any): any {
  return new Proxy(rawClient, {
    get(target, prop) {
      if (prop === "syncState") {
        return (...args: any[]) => target.syncStateImpl(...args);
      }
      if (prop === "proveBlock") {
        return async () => {
          const guard = await target.proveBlock();
          return guard;
        };
      }
      const val = target[prop];
      if (typeof val === "function") {
        const bound = val.bind(target);
        return (...args: any[]) => {
          const result = bound(...args);
          if (result && typeof result.then === "function") {
            return result.then((v: any) => (v === null ? undefined : v));
          }
          return result === null ? undefined : result;
        };
      }
      return val;
    },
  });
}

/**
 * Creates a platform-agnostic SDK wrapper for Node.js.
 * Provides the same interface as the browser's window.* types,
 * plus a `u64()` helper for platform-aware integer handling.
 */
function createNodeSdkWrapper(rawSdk: any): any {
  return {
    ...rawSdk,
    // u64: converts to the platform-appropriate type
    // Node.js napi uses f64 (number), browser uses BigInt
    u64: (val: number | bigint) =>
      typeof val === "bigint" ? Number(val) : val,
    // u64Array: converts an array of numbers to the platform-appropriate array type
    // Browser uses BigUint64Array, Node.js uses number[]
    u64Array: (vals: number[]) => vals,
  };
}

// ── Test helpers ──────────────────────────────────────────────────────

/**
 * Executes a transaction: execute → prove → submit → apply.
 * Works identically on both platforms.
 */
export async function executeAndApplyTransaction(
  client: any,
  sdk: any,
  accountId: any,
  transactionRequest: any,
  prover?: any
) {
  const result = await client.executeTransaction(accountId, transactionRequest);
  const proverToUse = prover ?? sdk.TransactionProver.newLocalProver();
  const proven = await client.proveTransaction(result, proverToUse);
  const submissionHeight = await client.submitProvenTransaction(proven, result);
  return await client.applyTransaction(result, submissionHeight);
}

/**
 * Waits for a transaction to be committed by polling syncState.
 */
export async function waitForTransaction(
  client: any,
  sdk: any,
  transactionId: string,
  maxWaitTime = 10000,
  delayInterval = 1000
) {
  let timeWaited = 0;
  while (true) {
    if (timeWaited >= maxWaitTime)
      throw new Error("Timeout waiting for transaction");
    await client.syncState();
    const uncommitted = await client.getTransactions(
      sdk.TransactionFilter.uncommitted()
    );
    const ids = uncommitted.map((tx: any) => tx.id().toHex());
    if (!ids.includes(transactionId)) break;
    await new Promise((r) => setTimeout(r, delayInterval));
    timeWaited += delayInterval;
  }
}

// ── Fixtures ──────────────────────────────────────────────────────────

export const test = base.extend<{
  client: any;
  sdk: any;
}>({
  client: async ({ page }, use, testInfo) => {
    const isNode = testInfo.project.name === "nodejs";

    if (isNode) {
      // For mock chain tests, create a mock client
      // For integration tests, create a real client
      // TODO: distinguish based on test tags or configuration
      const { client } = await createNodeMockClient();
      await use(client);
    } else {
      // Browser: use the page.evaluate-based proxy
      // For now, fall back to the old forEachTest pattern
      // TODO: implement browser client proxy properly
      await use(createBrowserClientProxy(page));
    }
  },

  sdk: async ({}, use, testInfo) => {
    const isNode = testInfo.project.name === "nodejs";

    if (isNode) {
      const rawSdk = loadNodeSdk();
      await use(createNodeSdkWrapper(rawSdk));
    } else {
      // Browser: SDK types are on window.* via forEachTest
      // TODO: provide a proper browser SDK wrapper
      await use({
        u64: (val: number | bigint) => BigInt(val),
        u64Array: (vals: number[]) => new BigUint64Array(vals.map(BigInt)),
      });
    }
  },
});

export { expect };
