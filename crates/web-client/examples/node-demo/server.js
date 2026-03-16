import { createRequire } from "module";
import express from "express";
import path from "path";
import fs from "fs";
import { fileURLToPath } from "url";

// napi .node files must be loaded via require()
const require = createRequire(import.meta.url);

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const PORT = process.env.PORT || 3000;
const NODE_URL = process.env.MIDEN_NODE_URL || "http://localhost:57291";
const DATA_DIR = process.env.DATA_DIR || "./miden-data";
const DB_PATH = path.join(DATA_DIR, "store.db");
const KEYSTORE_PATH = path.join(DATA_DIR, "keystore");

// Path to the compiled napi binary — build with:
//   cargo build -p miden-client-web --no-default-features --features nodejs,testing --release
// Then copy target/release/libmiden_client_web.dylib (macOS) or .so (Linux) here as miden.node
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const NATIVE_MODULE_PATH =
  process.env.MIDEN_MODULE_PATH || path.join(__dirname, "miden.node");

// ---------------------------------------------------------------------------
// Initialize Miden SDK
// ---------------------------------------------------------------------------

let sdk;
try {
  sdk = require(NATIVE_MODULE_PATH);
} catch (err) {
  console.error(`Failed to load native module from ${NATIVE_MODULE_PATH}`);
  console.error(
    "Build it with: cargo build -p miden-client-web --no-default-features --features nodejs,testing --release"
  );
  console.error(
    "Then: cp target/release/libmiden_client_web.dylib examples/node-demo/miden.node"
  );
  process.exit(1);
}

const client = new sdk.WebClient();

async function initClient() {
  // Ensure data directories exist
  fs.mkdirSync(DATA_DIR, { recursive: true });
  fs.mkdirSync(KEYSTORE_PATH, { recursive: true });

  console.log(`Connecting to Miden node at ${NODE_URL}`);
  console.log(`Store: ${DB_PATH}`);
  console.log(`Keystore: ${KEYSTORE_PATH}`);

  await client.createClient(
    NODE_URL,
    null,
    null,
    DB_PATH,
    KEYSTORE_PATH,
    false
  );
  console.log("Miden client initialized");

  // Initial sync
  await client.syncStateImpl();
  const height = await client.getSyncHeight();
  console.log(`Synced to block ${height}`);
}

// ---------------------------------------------------------------------------
// Express app
// ---------------------------------------------------------------------------

const app = express();
app.use(express.json());

// Health check
app.get("/health", async (_req, res) => {
  try {
    const height = await client.getSyncHeight();
    res.json({ status: "ok", syncHeight: Number(height) });
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

// Sync with the network
app.post("/sync", async (_req, res) => {
  try {
    await client.syncStateImpl();
    const height = await client.getSyncHeight();
    res.json({ syncHeight: Number(height) });
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

// List all accounts
app.get("/accounts", async (_req, res) => {
  try {
    const headers = await client.getAccounts();
    const result = headers.map((h) => ({
      id: h.id().toString(),
      nonce: h.nonce().toString(),
    }));
    res.json(result);
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

// Get single account with balance details
app.get("/accounts/:id", async (req, res) => {
  try {
    const accountId = sdk.AccountId.fromHex(req.params.id);
    const account = await client.getAccount(accountId);
    if (!account) {
      return res.status(404).json({ error: "Account not found" });
    }

    const assets = account
      .vault()
      .fungibleAssets()
      .map((asset) => ({
        faucetId: asset.faucetId().toString(),
        amount: asset.amount().toString(),
      }));

    res.json({
      id: account.id().toString(),
      nonce: account.nonce().toString(),
      isPublic: account.isPublic(),
      isFaucet: account.isFaucet(),
      isUpdatable: account.isUpdatable(),
      assets,
    });
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

// Create a new wallet
app.post("/wallets", async (req, res) => {
  try {
    const { storageMode = "private", mutable = true } = req.body || {};

    const mode =
      storageMode === "public"
        ? sdk.AccountStorageMode.public()
        : sdk.AccountStorageMode.private();

    const wallet = await client.newWallet(
      mode,
      mutable,
      sdk.AuthScheme.AuthRpoFalcon512
    );

    res.status(201).json({
      id: wallet.id().toString(),
      nonce: wallet.nonce().toString(),
      isPublic: wallet.isPublic(),
      isUpdatable: wallet.isUpdatable(),
    });
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

// Create a new faucet
app.post("/faucets", async (req, res) => {
  try {
    const {
      storageMode = "private",
      tokenSymbol = "TOK",
      decimals = 8,
      maxSupply = 1000000000,
    } = req.body || {};

    const mode =
      storageMode === "public"
        ? sdk.AccountStorageMode.public()
        : sdk.AccountStorageMode.private();

    const faucet = await client.newFaucet(
      mode,
      false,
      tokenSymbol,
      decimals,
      maxSupply,
      sdk.AuthScheme.AuthRpoFalcon512
    );

    res.status(201).json({
      id: faucet.id().toString(),
      nonce: faucet.nonce().toString(),
      tokenSymbol,
      decimals,
      maxSupply,
    });
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

// Mint tokens to a wallet from a faucet
app.post("/mint", async (req, res) => {
  try {
    const { walletId, faucetId, amount = 1000 } = req.body || {};

    if (!walletId || !faucetId) {
      return res
        .status(400)
        .json({ error: "walletId and faucetId are required" });
    }

    const wallet = sdk.AccountId.fromHex(walletId);
    const faucet = sdk.AccountId.fromHex(faucetId);

    const mintRequest = await client.newMintTransactionRequest(
      wallet,
      faucet,
      sdk.NoteType.Public,
      amount
    );

    const txId = await client.submitNewTransaction(faucet, mintRequest);

    res.status(201).json({
      transactionId: txId.toHex(),
      message: `Minting ${amount} tokens from ${faucetId} to ${walletId}`,
    });
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

// Send tokens between accounts
app.post("/send", async (req, res) => {
  try {
    const { senderId, targetId, faucetId, amount = 100 } = req.body || {};

    if (!senderId || !targetId || !faucetId) {
      return res
        .status(400)
        .json({ error: "senderId, targetId, and faucetId are required" });
    }

    const sender = sdk.AccountId.fromHex(senderId);
    const target = sdk.AccountId.fromHex(targetId);
    const faucet = sdk.AccountId.fromHex(faucetId);

    const sendRequest = await client.newSendTransactionRequest(
      sender,
      target,
      faucet,
      sdk.NoteType.Public,
      amount,
      null,
      null
    );

    const txId = await client.submitNewTransaction(sender, sendRequest);

    res.status(201).json({
      transactionId: txId.toHex(),
      message: `Sending ${amount} tokens from ${senderId} to ${targetId}`,
    });
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

// List transactions
app.get("/transactions", async (_req, res) => {
  try {
    const txs = await client.getTransactions(sdk.TransactionFilter.all());
    const result = txs.map((tx) => ({
      id: tx.id().toHex(),
      accountId: tx.accountId().toString(),
    }));
    res.json(result);
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

// ---------------------------------------------------------------------------
// Start
// ---------------------------------------------------------------------------

async function main() {
  await initClient();

  app.listen(PORT, () => {
    console.log(`\nMiden Wallet API running on http://localhost:${PORT}`);
    console.log("\nEndpoints:");
    console.log("  GET  /health          - Health check + sync height");
    console.log("  POST /sync            - Sync with the network");
    console.log("  GET  /accounts        - List all accounts");
    console.log("  GET  /accounts/:id    - Get account details + balances");
    console.log("  POST /wallets         - Create a new wallet");
    console.log("  POST /faucets         - Create a new faucet");
    console.log("  POST /mint            - Mint tokens to a wallet");
    console.log("  POST /send            - Send tokens between accounts");
    console.log("  GET  /transactions    - List all transactions");
    console.log(`\nExample flow:`);
    console.log(`  curl localhost:${PORT}/health`);
    console.log(`  curl -X POST localhost:${PORT}/wallets`);
    console.log(
      `  curl -X POST localhost:${PORT}/faucets -H 'Content-Type: application/json' -d '{"tokenSymbol":"DAG"}'`
    );
    console.log(
      `  curl -X POST localhost:${PORT}/mint -H 'Content-Type: application/json' -d '{"walletId":"<WALLET_ID>","faucetId":"<FAUCET_ID>","amount":5000}'`
    );
    console.log(`  curl -X POST localhost:${PORT}/sync`);
    console.log(`  curl localhost:${PORT}/accounts/<WALLET_ID>`);
  });
}

main().catch((err) => {
  console.error("Failed to start:", err);
  process.exit(1);
});
