/**
 * Shared Playwright fixtures for both browser and Node.js tests.
 *
 * Provides an `ops` fixture with platform-aware SDK operations:
 * - Browser (chromium/webkit): executes via page.evaluate() (uses browser global setup)
 * - Node.js: calls the napi SDK directly
 *
 * Test files use `ops.createNewWallet(params)` without knowing the platform.
 */
import { test as browserTest } from "../playwright.global.setup.ts";
import { createRequire } from "module";
import path from "path";
import fs from "fs";
import os from "os";
import * as sharedOps from "./ops.ts";

// ── Node.js SDK (loaded lazily, only for nodejs project) ──────────────
const require = createRequire(import.meta.url);

let _nodeSdk: any = null;
function getNodeSdk() {
  if (_nodeSdk) return _nodeSdk;

  const repoRoot = path.resolve(import.meta.dirname, "..", "..", "..", "..");
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
    `napi module not found. Build with: cargo build -p miden-client-web --no-default-features --features nodejs,testing --release --target ${target}`
  );
}

let _nodeTestCounter = 0;
async function createNodeMockClient() {
  const sdk = getNodeSdk();
  const tmpDir = path.join(
    os.tmpdir(),
    `miden-test-${process.pid}-${++_nodeTestCounter}`
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
  return { client, sdk };
}

// ── Operations interface ──────────────────────────────────────────────

export interface Ops {
  createNewWallet(params: {
    storageMode: string;
    mutable: boolean;
  }): Promise<any>;
  createNewFaucet(params: {
    storageMode: string;
    nonFungible: boolean;
    tokenSymbol: string;
    decimals: number;
    maxSupply: number;
  }): Promise<any>;
  mockChainMintAndConsume(): Promise<string>;
}

// ── Fixture ───────────────────────────────────────────────────────────
// Extends the browser test (which has the forEachTest auto-fixture that
// loads the SDK into window.* and creates window.client). For Node.js,
// the forEachTest fixture is a no-op since there's no page navigation.

export const test = browserTest.extend<{ ops: Ops }>({
  ops: async ({ page }, use, testInfo) => {
    const isNode = testInfo.project.name === "nodejs";

    if (isNode) {
      const { client, sdk } = await createNodeMockClient();
      await use({
        createNewWallet: (params) =>
          sharedOps.createNewWallet(client, sdk, params),
        createNewFaucet: (params) =>
          sharedOps.createNewFaucet(client, sdk, params),
        mockChainMintAndConsume: () =>
          sharedOps.mockChainMintAndConsume(client, sdk),
      });
    } else {
      // Browser: page is already set up by forEachTest (SDK loaded, window.client created)
      await use({
        createNewWallet: (params) =>
          page.evaluate(async (p) => {
            const mode = window.AccountStorageMode.tryFromStr(p.storageMode);
            const wallet = await window.client.newWallet(
              mode,
              p.mutable,
              window.AuthScheme.AuthRpoFalcon512
            );
            return {
              id: wallet.id().toString(),
              nonce: wallet.nonce().toString(),
              vaultCommitment: wallet.vault().root().toHex(),
              storageCommitment: wallet.storage().commitment().toHex(),
              codeCommitment: wallet.code().commitment().toHex(),
              isFaucet: wallet.isFaucet(),
              isRegularAccount: wallet.isRegularAccount(),
              isUpdatable: wallet.isUpdatable(),
              isPublic: wallet.isPublic(),
              isPrivate: wallet.isPrivate(),
              isNetwork: wallet.isNetwork(),
              isIdPublic: wallet.id().isPublic(),
              isIdPrivate: wallet.id().isPrivate(),
              isIdNetwork: wallet.id().isNetwork(),
              isNew: wallet.isNew(),
            };
          }, params),

        createNewFaucet: (params) =>
          page.evaluate(async (p) => {
            const mode = window.AccountStorageMode.tryFromStr(p.storageMode);
            const faucet = await window.client.newFaucet(
              mode,
              p.nonFungible,
              p.tokenSymbol,
              p.decimals,
              BigInt(p.maxSupply),
              window.AuthScheme.AuthRpoFalcon512
            );
            return {
              id: faucet.id().toString(),
              nonce: faucet.nonce().toString(),
              vaultCommitment: faucet.vault().root().toHex(),
              storageCommitment: faucet.storage().commitment().toHex(),
              codeCommitment: faucet.code().commitment().toHex(),
              isFaucet: faucet.isFaucet(),
              isRegularAccount: faucet.isRegularAccount(),
              isUpdatable: faucet.isUpdatable(),
              isPublic: faucet.isPublic(),
              isPrivate: faucet.isPrivate(),
              isNetwork: faucet.isNetwork(),
              isIdPublic: faucet.id().isPublic(),
              isIdPrivate: faucet.id().isPrivate(),
              isIdNetwork: faucet.id().isNetwork(),
              isNew: faucet.isNew(),
            };
          }, params),

        mockChainMintAndConsume: () =>
          page.evaluate(async () => {
            const client = await window.MockWasmWebClient.createClient();
            await client.syncState();
            const wallet = await client.newWallet(
              window.AccountStorageMode.private(),
              true,
              window.AuthScheme.AuthRpoFalcon512
            );
            const faucet = await client.newFaucet(
              window.AccountStorageMode.private(),
              false,
              "DAG",
              8,
              BigInt(10000000),
              window.AuthScheme.AuthRpoFalcon512
            );
            const mintRequest = await client.newMintTransactionRequest(
              wallet.id(),
              faucet.id(),
              window.NoteType.Public,
              BigInt(1000)
            );
            const mintTxId = await client.submitNewTransaction(
              faucet.id(),
              mintRequest
            );
            await client.proveBlock();
            await client.syncState();
            const [mintRecord] = await client.getTransactions(
              window.TransactionFilter.ids([mintTxId])
            );
            const mintedNoteId = mintRecord
              .outputNotes()
              .notes()[0]
              .id()
              .toString();
            const noteRecord = await client.getInputNote(mintedNoteId);
            const note = noteRecord.toNote();
            const consumeRequest = client.newConsumeTransactionRequest([note]);
            await client.submitNewTransaction(wallet.id(), consumeRequest);
            await client.proveBlock();
            await client.syncState();
            const account = await client.getAccount(wallet.id());
            return account.vault().getBalance(faucet.id()).toString();
          }),
      });
    }
  },
});

export { expect } from "@playwright/test";
