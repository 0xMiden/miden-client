//@ts-nocheck
import { test as base, TestInfo } from "@playwright/test";

// Unique per test run so concurrent suites don't share IndexedDB stores.
export const RUN_ID = crypto.randomUUID().slice(0, 8);

function generateStoreName(testInfo: TestInfo): string {
  return `test_${RUN_ID}_${testInfo.testId}`;
}

const TEST_SERVER_PORT = 8080;
const MIDEN_NODE_PORT = 57291;
const REMOTE_TX_PROVER_PORT = 50051;

// Check if running against localhost (vs devnet/testnet)
export function isLocalhost(): boolean {
  if (process.env.TEST_MIDEN_RPC_URL) {
    return process.env.TEST_MIDEN_RPC_URL.includes("localhost");
  }
  const network = process.env.TEST_MIDEN_NETWORK?.toLowerCase();
  return !network || network === "localhost";
}

// Determine RPC URL from environment or default to localhost
export function getRpcUrl(): string {
  if (process.env.TEST_MIDEN_RPC_URL) {
    return process.env.TEST_MIDEN_RPC_URL;
  }

  const network = process.env.TEST_MIDEN_NETWORK?.toLowerCase();
  switch (network) {
    case "devnet":
      return "https://rpc.devnet.miden.io";
    case "testnet":
      return "https://rpc.testnet.miden.io";
    case "localhost":
    default:
      return `http://localhost:${MIDEN_NODE_PORT}`;
  }
}

// Determine remote prover URL from environment or default based on network.
// Returns undefined if remote proving is not requested (REMOTE_PROVER not set).
export function getProverUrl(): string | undefined {
  if (process.env.TEST_MIDEN_PROVER_URL) {
    return process.env.TEST_MIDEN_PROVER_URL;
  }

  if (!process.env.REMOTE_PROVER) {
    return undefined;
  }

  const network = process.env.TEST_MIDEN_NETWORK?.toLowerCase();
  switch (network) {
    case "devnet":
      return "https://tx-prover.devnet.miden.io";
    case "testnet":
      return "https://tx-prover.testnet.miden.io";
    case "localhost":
    default:
      return `http://localhost:${REMOTE_TX_PROVER_PORT}`;
  }
}

export const test = base.extend<{ forEachTest: void }>({
  forEachTest: [
    async ({ page }, use, testInfo) => {
      const storeName = generateStoreName(testInfo);
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

      await page.goto("http://localhost:8080");

      await page.evaluate(
        async ({ rpcUrl, proverUrl, storeName }) => {
          // Import the sdk classes and attach them
          // to the window object for testing
          const sdkExports = await import("./index.js");
          for (const [key, value] of Object.entries(sdkExports)) {
            window[key] = value;
          }
          const client = await window.WasmWebClient.createClient(
            rpcUrl,
            undefined,
            undefined,
            storeName
          );
          window.rpcUrl = rpcUrl;
          window.storeName = storeName;

          window.client = client;

          // Create a namespace for helper functions
          window.helpers = window.helpers || {};

          // Add the remote prover url to window
          window.remoteProverUrl = proverUrl;
          if (window.remoteProverUrl) {
            window.remoteProverInstance =
              window.TransactionProver.newRemoteProver(
                window.remoteProverUrl,
                BigInt(20_000)
              );
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
            const client = await WasmWebClient.createClient(
              rpcUrl,
              undefined,
              initSeed,
              window.storeName
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
        },
        {
          rpcUrl: getRpcUrl(),
          proverUrl: getProverUrl() ?? null,
          storeName,
        }
      );
      await use();
    },
    { auto: true },
  ],
});

export default test;
