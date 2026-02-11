/**
 * Node.js integration test for the Miden web-client.
 *
 * Exercises the full client lifecycle against a live node:
 *   1. Create client & sync
 *   2. Create wallet and faucet accounts
 *   3. Mint tokens (faucet -> wallet)
 *   4. Consume minted note (wallet receives tokens)
 *   5. Check wallet balance
 *   6. Send tokens (wallet -> wallet2)
 *   7. Consume sent note (wallet2 receives tokens)
 *   8. Verify final balances
 *
 * Usage: node integration.mjs [rpc-url]
 *   Default RPC: https://rpc.devnet.miden.io
 */

import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { existsSync, unlinkSync } from "node:fs";
import { loadSdk } from "./load-sdk.mjs";

const RPC_URL = process.argv[2] || "https://rpc.devnet.miden.io";

// Set SQLite DB path for node-store (cleaned up at start)
const SQLITE_PATH = resolve(
  dirname(fileURLToPath(import.meta.url)),
  "test-miden-store.sqlite"
);
if (existsSync(SQLITE_PATH)) unlinkSync(SQLITE_PATH);
globalThis.__MIDEN_STORE_PATH = SQLITE_PATH;

// ─── Helpers ────────────────────────────────────────────────────────────────

function log(step, msg) {
  console.log(`[${step}] ${msg}`);
}

function assert(condition, msg) {
  if (!condition) {
    throw new Error(`Assertion failed: ${msg}`);
  }
}

async function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

/**
 * Wait for a transaction to be committed by polling syncState + getTransactions.
 */
async function waitForTransaction(
  client,
  sdk,
  transactionIdHex,
  maxWaitMs = 60000
) {
  const startTime = Date.now();
  while (true) {
    if (Date.now() - startTime > maxWaitMs) {
      throw new Error(
        `Timeout waiting for transaction ${transactionIdHex} after ${maxWaitMs}ms`
      );
    }
    await client.syncState();
    const uncommitted = await client.getTransactions(
      sdk.TransactionFilter.uncommitted()
    );
    const uncommittedIds = uncommitted.map((tx) => tx.id().toHex());
    if (!uncommittedIds.includes(transactionIdHex)) {
      return;
    }
    await sleep(2000);
  }
}

/**
 * Execute, prove, submit, and apply a transaction (mirrors browser helpers.executeAndApplyTransaction).
 */
async function executeAndApplyTransaction(
  client,
  sdk,
  accountId,
  transactionRequest
) {
  const result = await client.executeTransaction(accountId, transactionRequest);
  const prover = sdk.TransactionProver.newLocalProver();
  const proven = await client.proveTransaction(result, prover);
  const submissionHeight = await client.submitProvenTransaction(proven, result);
  return await client.applyTransaction(result, submissionHeight);
}

// ─── Main ───────────────────────────────────────────────────────────────────

async function main() {
  console.log("=".repeat(60));
  console.log("Miden Web Client — Node.js Integration Test");
  console.log(`RPC: ${RPC_URL}`);
  console.log(`Node: ${process.version}`);
  console.log("=".repeat(60));

  // ── 1. Load SDK ─────────────────────────────────────────────────────────
  log("1", "Loading WASM SDK...");
  const sdk = await loadSdk();
  log("1", `SDK loaded (${Object.keys(sdk).length} exports)`);

  // ── 2. Create client ────────────────────────────────────────────────────
  log("2", "Creating WebClient...");
  const client = await sdk.WebClient.createClient(RPC_URL);
  log("2", "WebClient created");

  // ── 3. Initial sync ─────────────────────────────────────────────────────
  log("3", "Syncing state...");
  let syncSummary = await client.syncState();
  log("3", `Synced to block ${syncSummary.blockNum()}`);

  // ── 4. Create wallet ────────────────────────────────────────────────────
  log("4", "Creating wallet...");
  const wallet = await client.newWallet(
    sdk.AccountStorageMode.private(),
    true, // mutable
    0 // AuthScheme: Falcon512
  );
  const walletId = wallet.id();
  log("4", `Wallet created: ${walletId.toString()}`);
  assert(wallet.isRegularAccount(), "wallet should be a regular account");
  assert(!wallet.isFaucet(), "wallet should not be a faucet");

  // ── 5. Create faucet ───────────────────────────────────────────────────
  log("5", "Creating faucet...");
  const faucet = await client.newFaucet(
    sdk.AccountStorageMode.private(),
    false, // not non-fungible
    "NOD", // token symbol
    8, // decimals
    BigInt(10000000), // max supply
    0 // AuthScheme: Falcon512
  );
  const faucetId = faucet.id();
  log("5", `Faucet created: ${faucetId.toString()}`);
  assert(faucet.isFaucet(), "faucet should be a faucet");

  // ── 6. Sync after account creation ──────────────────────────────────────
  log("6", "Syncing after account creation...");
  syncSummary = await client.syncState();
  log("6", `Synced to block ${syncSummary.blockNum()}`);

  // ── 7. Mint tokens from faucet to wallet ────────────────────────────────
  log("7", "Minting 1000 tokens to wallet...");
  const mintRequest = client.newMintTransactionRequest(
    walletId,
    faucetId,
    sdk.NoteType.Public,
    BigInt(1000)
  );

  const mintUpdate = await executeAndApplyTransaction(
    client,
    sdk,
    faucetId,
    mintRequest
  );
  const mintTxId = mintUpdate.executedTransaction().id().toHex();
  log("7", `Mint tx submitted: ${mintTxId}`);

  const mintedNote = mintUpdate.executedTransaction().outputNotes().notes()[0];
  const mintedNoteId = mintedNote.id().toString();
  log("7", `Minted note: ${mintedNoteId}`);

  // ── 8. Wait for mint tx to be committed ─────────────────────────────────
  log("8", "Waiting for mint tx to be committed...");
  await waitForTransaction(client, sdk, mintTxId);
  log("8", "Mint tx committed");

  // ── 9. Consume minted note (wallet receives tokens) ─────────────────────
  log("9", "Consuming minted note...");
  const inputNoteRecord = await client.getInputNote(mintedNoteId);
  assert(inputNoteRecord, `Note ${mintedNoteId} should be in local store`);
  const note = inputNoteRecord.toNote();

  const consumeRequest = client.newConsumeTransactionRequest([note]);
  const consumeUpdate = await executeAndApplyTransaction(
    client,
    sdk,
    walletId,
    consumeRequest
  );
  const consumeTxId = consumeUpdate.executedTransaction().id().toHex();
  log("9", `Consume tx submitted: ${consumeTxId}`);

  // ── 10. Wait for consume tx ─────────────────────────────────────────────
  log("10", "Waiting for consume tx to be committed...");
  await waitForTransaction(client, sdk, consumeTxId);
  log("10", "Consume tx committed");

  // ── 11. Check wallet balance ────────────────────────────────────────────
  log("11", "Checking wallet balance...");
  const walletAccount = await client.getAccount(walletId);
  assert(walletAccount, "wallet account should exist");
  const balance = walletAccount.vault().getBalance(faucetId);
  log("11", `Wallet balance: ${balance} (expected: 1000)`);
  assert(balance === BigInt(1000), `expected balance 1000, got ${balance}`);

  // ── 12. Create second wallet ────────────────────────────────────────────
  log("12", "Creating second wallet...");
  const wallet2 = await client.newWallet(
    sdk.AccountStorageMode.private(),
    true,
    0
  );
  const wallet2Id = wallet2.id();
  log("12", `Wallet2 created: ${wallet2Id.toString()}`);

  // ── 13. Send tokens wallet -> wallet2 ───────────────────────────────────
  log("13", "Sending 100 tokens from wallet to wallet2...");
  const sendRequest = client.newSendTransactionRequest(
    walletId,
    wallet2Id,
    faucetId,
    sdk.NoteType.Public,
    BigInt(100),
    undefined, // recallHeight
    null // timelockHeight
  );
  const sendUpdate = await executeAndApplyTransaction(
    client,
    sdk,
    walletId,
    sendRequest
  );
  const sendTxId = sendUpdate.executedTransaction().id().toHex();
  const sentNoteId = sendUpdate
    .executedTransaction()
    .outputNotes()
    .notes()[0]
    .id()
    .toString();
  log("13", `Send tx submitted: ${sendTxId}, note: ${sentNoteId}`);

  // ── 14. Wait for send tx ────────────────────────────────────────────────
  log("14", "Waiting for send tx to be committed...");
  await waitForTransaction(client, sdk, sendTxId);
  log("14", "Send tx committed");

  // ── 15. Consume sent note (wallet2 receives tokens) ─────────────────────
  log("15", "Consuming sent note on wallet2...");
  const sentNoteRecord = await client.getInputNote(sentNoteId);
  assert(sentNoteRecord, `Note ${sentNoteId} should be in local store`);
  const sentNote = sentNoteRecord.toNote();

  const consumeRequest2 = client.newConsumeTransactionRequest([sentNote]);
  const consumeUpdate2 = await executeAndApplyTransaction(
    client,
    sdk,
    wallet2Id,
    consumeRequest2
  );
  const consumeTxId2 = consumeUpdate2.executedTransaction().id().toHex();
  log("15", `Consume tx submitted: ${consumeTxId2}`);

  // ── 16. Wait for consume tx ─────────────────────────────────────────────
  log("16", "Waiting for wallet2 consume tx...");
  await waitForTransaction(client, sdk, consumeTxId2);
  log("16", "Consume tx committed");

  // ── 17. Verify final balances ───────────────────────────────────────────
  log("17", "Verifying final balances...");
  const finalWallet = await client.getAccount(walletId);
  const finalWallet2 = await client.getAccount(wallet2Id);

  const walletBalance = finalWallet.vault().getBalance(faucetId);
  const wallet2Balance = finalWallet2.vault().getBalance(faucetId);

  log("17", `Wallet  balance: ${walletBalance} (expected: 900)`);
  log("17", `Wallet2 balance: ${wallet2Balance} (expected: 100)`);

  assert(
    walletBalance === BigInt(900),
    `wallet expected 900, got ${walletBalance}`
  );
  assert(
    wallet2Balance === BigInt(100),
    `wallet2 expected 100, got ${wallet2Balance}`
  );

  // ── 18. List accounts ───────────────────────────────────────────────────
  log("18", "Listing accounts...");
  const accounts = await client.getAccounts();
  log("18", `Total accounts: ${accounts.length}`);
  for (const header of accounts) {
    log("18", `  - ${header.id().toString()}`);
  }

  // ── Cleanup ─────────────────────────────────────────────────────────────
  client.terminate();

  console.log("\n" + "=".repeat(60));
  console.log("ALL TESTS PASSED");
  console.log("=".repeat(60));
}

main().catch((err) => {
  console.error("\nTEST FAILED:", err);
  process.exit(1);
});
