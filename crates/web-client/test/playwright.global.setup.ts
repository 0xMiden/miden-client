//@ts-nocheck
import { test as base } from "@playwright/test";
import { MockWebClient } from "../js";

const TEST_SERVER_PORT = 8080;
const MIDEN_NODE_PORT = 57291;
const REMOTE_TX_PROVER_PORT = 50051;

export const test = base.extend<{ forEachTest: void }>({
  forEachTest: [
    async ({ page }, use) => {
      console.log("[SETUP] Starting test setup...");

      page.on("console", (msg) => {
        if (msg.type() === "debug") {
          console.log(`PAGE DEBUG: ${msg.text()}`);
        }
      });

      page.on("pageerror", (err) => {
        console.error("PAGE ERROR:", err);
      });

      page.on("error", (err) => {
        console.error("PUPPETEER ERROR:", err);
      });

      console.log("[SETUP] Navigating to http://localhost:8080...");
      await page.goto("http://localhost:8080");
      console.log("[SETUP] Navigation complete. Starting page.evaluate...");

      await page.evaluate(
        async ({ MIDEN_NODE_PORT, remoteProverPort }) => {
          console.log("[SETUP-PAGE] Inside page.evaluate, starting SDK import...");
          // Import the sdk classes and attach them
          // to the window object for testing
          const sdkExports = await import("./index.js");
          console.log("[SETUP-PAGE] SDK imported, attaching exports to window...");
          for (const [key, value] of Object.entries(sdkExports)) {
            window[key] = value;
          }
          console.log("[SETUP-PAGE] Exports attached. Keys:", Object.keys(sdkExports).join(", "));

          let rpcUrl = `http://localhost:${MIDEN_NODE_PORT}`;
          let proverUrl = remoteProverPort
            ? `http://localhost:${remoteProverPort}`
            : undefined;
          console.log("[SETUP-PAGE] Creating WebClient with rpcUrl:", rpcUrl);
          const client = await window.WebClient.createClient(
            rpcUrl,
            undefined,
            undefined,
            "tests"
          );
          console.log("[SETUP-PAGE] WebClient created successfully");
          window.rpcUrl = rpcUrl;

          window.client = client;
          console.log("[SETUP-PAGE] Client assigned to window.client");

          // Create a namespace for helper functions
          window.helpers = window.helpers || {};
          console.log("[SETUP-PAGE] Helpers namespace created");

          // Add the remote prover url to window
          window.remoteProverUrl = proverUrl;
          console.log("[SETUP-PAGE] Remote prover URL:", proverUrl || "not set");
          if (window.remoteProverUrl) {
            console.log("[SETUP-PAGE] Creating remote prover instance...");
            window.remoteProverInstance =
              window.TransactionProver.newRemoteProver(
                window.remoteProverUrl,
                BigInt(10_000)
              );
            console.log("[SETUP-PAGE] Remote prover instance created");
          }

          window.helpers.waitForTransaction = async (
            transactionId,
            maxWaitTime = 10000,
            delayInterval = 1000
          ) => {
            const client = window.client;
            let timeWaited = 0;
            while (true) {
              if (timeWaited >= maxWaitTime) {
                throw new Error("Timeout waiting for transaction");
              }
              await client.syncState();
              const uncommittedTransactions = await client.getTransactions(
                window.TransactionFilter.uncommitted()
              );
              let uncommittedTransactionIds = uncommittedTransactions.map(
                (transaction) => transaction.id().toHex()
              );
              if (!uncommittedTransactionIds.includes(transactionId)) {
                break;
              }
              await new Promise((r) => setTimeout(r, delayInterval));
              timeWaited += delayInterval;
            }
          };

          window.helpers.executeAndApplyTransaction = async (
            accountId,
            transactionRequest,
            prover
          ) => {
            const client = window.client;
            const result = await client.executeTransaction(
              accountId,
              transactionRequest
            );

            const useRemoteProver =
              prover != null && window.remoteProverUrl != null;
            const proverToUse = useRemoteProver
              ? window.TransactionProver.newRemoteProver(
                  window.remoteProverUrl,
                  null
                )
              : window.TransactionProver.newLocalProver();

            const proven = await client.proveTransaction(result, proverToUse);
            const submissionHeight = await client.submitProvenTransaction(
              proven,
              result
            );
            return await client.applyTransaction(result, submissionHeight);
          };

          window.helpers.waitForBlocks = async (amountOfBlocks) => {
            const client = window.client;
            let currentBlock = await client.getSyncHeight();
            let finalBlock = currentBlock + amountOfBlocks;
            console.log(
              `Current block: ${currentBlock}, waiting for ${amountOfBlocks} blocks...`
            );
            while (true) {
              let syncSummary = await client.syncState();
              console.log(
                `Synced to block ${syncSummary.blockNum()} (syncing until ${finalBlock})`
              );
              if (syncSummary.blockNum() >= finalBlock) {
                return;
              }
              await new Promise((r) => setTimeout(r, 1000));
            }
          };

          window.helpers.refreshClient = async (initSeed) => {
            const client = await WebClient.createClient(
              rpcUrl,
              undefined,
              initSeed
            );
            window.client = client;
            await window.client.syncState();
          };

          window.helpers.parseNetworkId = (networkId) => {
            const map = {
              mm: window.NetworkId.mainnet(),
              mtst: window.NetworkId.testnet(),
              mdev: window.NetworkId.devnet(),
            };
            let parsedNetworkId = map[networkId];
            if (parsedNetworkId === undefined) {
              try {
                parsedNetworkId = window.NetworkId.custom(networkId);
              } catch (error) {
                throw new Error(
                  `Invalid network ID: ${networkId}. Expected one of: ${Object.keys(map).join(", ")}, or a valid custom network ID`
                );
              }
            }
            return parsedNetworkId;
          };

          console.log("[SETUP-PAGE] All helper functions defined");
          console.log("[SETUP-PAGE] Setup complete!");
        },
        {
          MIDEN_NODE_PORT,
          remoteProverPort: process.env.REMOTE_PROVER
            ? REMOTE_TX_PROVER_PORT
            : null,
        }
      );
      console.log("[SETUP] page.evaluate completed successfully");
      console.log("[SETUP] Calling use() to run the test...");
      await use();
      console.log("[SETUP] Test completed, cleanup starting...");
    },
    { auto: true },
  ],
});

export default test;
