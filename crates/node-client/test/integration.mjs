/**
 * Integration test for the node-client native addon.
 *
 * Requires a running Miden test node (e.g. `make start-node-background`).
 * The RPC URL defaults to http://localhost:57291 (the local test node).
 *
 * Usage:
 *   node crates/node-client/test/integration.mjs [rpc_url]
 */

import { createRequire } from "module";
import { mkdtempSync, rmSync, existsSync } from "fs";
import { tmpdir } from "os";
import { join } from "path";

const require = createRequire(import.meta.url);
const sdk = require("../index.js");

const { NodeClient } = sdk;

const RPC_URL = process.argv[2] || "http://localhost:57291";
const MINT_AMOUNT = 1000n;
const SEND_AMOUNT = 500n;

// Utilities
// ================================================================================================

function assert(condition, message) {
  if (!condition) {
    throw new Error(`ASSERTION FAILED: ${message}`);
  }
}

function assertEqual(actual, expected, label) {
  if (actual !== expected) {
    throw new Error(
      `ASSERTION FAILED [${label}]: expected ${expected}, got ${actual}`
    );
  }
}

function log(step, message) {
  console.log(`[${step}] ${message}`);
}

/**
 * Syncs the client repeatedly until a new block appears or maxAttempts is reached.
 * Returns the last SyncSummary.
 */
function waitForBlocks(client, blocks, maxAttempts = 60, delayMs = 3000) {
  const startHeight = client.getSyncHeight();
  const targetHeight = startHeight + blocks;
  log("SYNC", `Waiting for block ${targetHeight} (current: ${startHeight})`);

  for (let i = 0; i < maxAttempts; i++) {
    const summary = client.syncState();
    const currentHeight = summary.blockNum();
    if (currentHeight >= targetHeight) {
      log("SYNC", `Reached block ${currentHeight}`);
      return summary;
    }
    // Sleep synchronously (this is a test script)
    Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, delayMs);
  }
  throw new Error(
    `Timed out waiting for block ${targetHeight} after ${maxAttempts} attempts`
  );
}

// Test
// ================================================================================================

async function main() {
  // Create temp directories for the test
  const tempDir = mkdtempSync(join(tmpdir(), "miden-node-client-test-"));
  const dbPath = join(tempDir, "client.db");
  const keysDir = join(tempDir, "keys");

  log("SETUP", `Temp dir: ${tempDir}`);
  log("SETUP", `RPC URL: ${RPC_URL}`);

  try {
    // 1. Create client
    log("1", "Creating client...");
    const client = NodeClient.createClient(RPC_URL, dbPath, keysDir);
    log("1", "Client created successfully");

    // 2. Initial sync
    log("2", "Initial sync...");
    let summary = client.syncState();
    log("2", `Synced to block ${summary.blockNum()}`);

    // 3. Create wallet
    log("3", "Creating wallet...");
    const wallet = client.newWallet("Private", false, "RpoFalcon512");
    const walletId = wallet.id();
    log("3", `Created wallet: ${walletId.toString()}`);
    assert(wallet.isRegularAccount(), "Wallet should be a regular account");
    assert(!wallet.isFaucet(), "Wallet should not be a faucet");

    // 4. Create faucet
    log("4", "Creating faucet...");
    const faucet = client.newFaucet(
      "Public",
      false,
      "TST",
      8,
      1_000_000n,
      "RpoFalcon512"
    );
    const faucetId = faucet.id();
    log("4", `Created faucet: ${faucetId.toString()}`);
    assert(faucet.isFaucet(), "Faucet should be a faucet");

    // 5. Sync to get accounts committed
    log("5", "Syncing to commit accounts...");
    waitForBlocks(client, 2);
    log("5", `Synced to block ${client.getSyncHeight()}`);

    // 6. Mint tokens
    log("6", `Minting ${MINT_AMOUNT} tokens...`);
    const mintRequest = client.newMintTransactionRequest(
      walletId,
      faucetId,
      "Public",
      MINT_AMOUNT
    );
    const mintTxId = client.submitNewTransaction(faucetId, mintRequest);
    log("6", `Mint tx submitted: ${mintTxId.toHex()}`);

    // 7. Wait for mint transaction to be committed
    log("7", "Waiting for mint transaction to commit...");
    waitForBlocks(client, 3);

    // 8. Check for committed input notes
    log("8", "Checking for committed input notes...");
    const committedNotes = client.getInputNotes("Committed");
    log("8", `Found ${committedNotes.length} committed input note(s)`);
    assert(
      committedNotes.length > 0,
      "Should have at least one committed note after mint"
    );

    // Get the minted note
    const mintedNote = committedNotes[0];
    const mintedNoteId = mintedNote.id();
    log("8", `Minted note ID: ${mintedNoteId.toString()}`);

    // 9. Test getInputNote
    log("9", "Testing getInputNote...");
    const fetchedNote = client.getInputNote(mintedNoteId);
    assert(fetchedNote !== null, "getInputNote should return the note");
    assertEqual(
      fetchedNote.id().toString(),
      mintedNoteId.toString(),
      "Note IDs should match"
    );

    // 10. Consume the minted note
    log("10", "Consuming minted note...");
    const note = mintedNote.toNote();
    const consumeRequest = client.newConsumeTransactionRequest([note]);
    const consumeTxId = client.submitNewTransaction(walletId, consumeRequest);
    log("10", `Consume tx submitted: ${consumeTxId.toHex()}`);

    // 11. Wait for consume transaction
    log("11", "Waiting for consume transaction to commit...");
    waitForBlocks(client, 3);

    // 12. Check wallet balance
    log("12", "Checking wallet balance...");
    const updatedWallet = client.getAccount(walletId);
    assert(updatedWallet !== null, "Wallet should exist");
    const balance = updatedWallet.vault().getBalance(faucetId);
    log("12", `Wallet balance: ${balance}`);
    assertEqual(
      balance,
      MINT_AMOUNT,
      "Wallet balance after consuming minted note"
    );

    // 13. Send tokens from wallet to faucet
    log("13", `Sending ${SEND_AMOUNT} tokens from wallet...`);
    const sendRequest = client.newSendTransactionRequest(
      walletId,
      faucetId,
      faucetId,
      "Public",
      SEND_AMOUNT
    );
    const sendTxId = client.submitNewTransaction(walletId, sendRequest);
    log("13", `Send tx submitted: ${sendTxId.toHex()}`);

    // 14. Wait for send transaction
    log("14", "Waiting for send transaction to commit...");
    waitForBlocks(client, 3);

    // 15. Verify final balance
    log("15", "Verifying final wallet balance...");
    const finalWallet = client.getAccount(walletId);
    assert(finalWallet !== null, "Wallet should exist after send");
    const finalBalance = finalWallet.vault().getBalance(faucetId);
    log("15", `Final wallet balance: ${finalBalance}`);
    assertEqual(
      finalBalance,
      MINT_AMOUNT - SEND_AMOUNT,
      "Wallet balance after sending"
    );

    // 16. Test transaction history
    log("16", "Checking transaction history...");
    const allTxs = client.getTransactions("All");
    log("16", `Total transactions: ${allTxs.length}`);
    assert(
      allTxs.length >= 3,
      "Should have at least 3 transactions (mint, consume, send)"
    );

    // 17. Test tags
    log("17", "Testing tag operations...");
    client.addTag("12345");
    const tags = client.listTags();
    assert(tags.includes("12345"), "Tag should be in list");
    client.removeTag("12345");
    const tagsAfter = client.listTags();
    assert(!tagsAfter.includes("12345"), "Tag should be removed");

    // 18. Test settings
    log("18", "Testing settings operations...");
    client.setSetting("test_key", Buffer.from("test_value"));
    const setting = client.getSetting("test_key");
    assert(setting !== null, "Setting should exist");
    assertEqual(setting.toString(), "test_value", "Setting value");
    const keys = client.listSettingKeys();
    assert(keys.includes("test_key"), "Key should be in list");
    client.removeSetting("test_key");
    const settingAfter = client.getSetting("test_key");
    assertEqual(settingAfter, null, "Setting should be removed");

    log("DONE", "All tests passed!");
  } finally {
    // Cleanup
    if (existsSync(tempDir)) {
      rmSync(tempDir, { recursive: true, force: true });
    }
  }
}

main().catch((err) => {
  console.error("Test failed:", err);
  process.exit(1);
});
