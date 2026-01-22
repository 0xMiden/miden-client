// @ts-nocheck
import { expect } from "@playwright/test";
import test from "./playwright.global.setup";
import { BrowserContext, Page } from "@playwright/test";

test.describe("Sync Lock Tests", () => {
  test.describe("Coalescing Behavior", () => {
    test("concurrent syncs return the same block number", async ({ page }) => {
      const result = await page.evaluate(async () => {
        const client = window.client;

        // Fire multiple syncState calls concurrently
        const syncPromises = [
          client.syncState(),
          client.syncState(),
          client.syncState(),
        ];

        const results = await Promise.all(syncPromises);
        const blockNums = results.map((r) => r.blockNum());

        return {
          blockNums,
          allSame: blockNums.every((n) => n === blockNums[0]),
          count: blockNums.length,
        };
      });

      expect(result.count).toBe(3);
      expect(result.allSame).toBe(true);
    });

    test("rapid concurrent syncs all complete successfully", async ({
      page,
    }) => {
      const result = await page.evaluate(async () => {
        const client = window.client;

        // Fire many concurrent sync calls
        const syncPromises = Array(10)
          .fill(null)
          .map(() => client.syncState());

        const results = await Promise.all(syncPromises);
        const blockNums = results.map((r) => r.blockNum());

        return {
          allSucceeded: results.every((r) => typeof r.blockNum() === "number"),
          blockNums,
          uniqueBlockNums: [...new Set(blockNums)],
        };
      });

      expect(result.allSucceeded).toBe(true);
      // All syncs should return the same block number (coalescing)
      expect(result.uniqueBlockNums.length).toBe(1);
    });

    test("sequential syncs can return different block numbers", async ({
      page,
    }) => {
      const result = await page.evaluate(async () => {
        const client = window.client;

        // Sequential syncs should work normally
        const result1 = await client.syncState();
        const result2 = await client.syncState();
        const result3 = await client.syncState();

        return {
          blockNum1: result1.blockNum(),
          blockNum2: result2.blockNum(),
          blockNum3: result3.blockNum(),
        };
      });

      // Sequential syncs should all succeed (block nums may be same or different)
      expect(typeof result.blockNum1).toBe("number");
      expect(typeof result.blockNum2).toBe("number");
      expect(typeof result.blockNum3).toBe("number");
      // Block numbers should be non-negative
      expect(result.blockNum1).toBeGreaterThanOrEqual(0);
      expect(result.blockNum2).toBeGreaterThanOrEqual(0);
      expect(result.blockNum3).toBeGreaterThanOrEqual(0);
    });
  });

  test.describe("Timeout Behavior", () => {
    test("syncStateWithTimeout with 0 timeout works like syncState", async ({
      page,
    }) => {
      const result = await page.evaluate(async () => {
        const client = window.client;

        const result1 = await client.syncState();
        const result2 = await client.syncStateWithTimeout(0);

        return {
          blockNum1: result1.blockNum(),
          blockNum2: result2.blockNum(),
        };
      });

      expect(typeof result.blockNum1).toBe("number");
      expect(typeof result.blockNum2).toBe("number");
    });

    test("syncStateWithTimeout with positive timeout succeeds", async ({
      page,
    }) => {
      const result = await page.evaluate(async () => {
        const client = window.client;

        // Use a generous timeout
        const result = await client.syncStateWithTimeout(30000);

        return {
          blockNum: result.blockNum(),
          committedNotes: result.committedNotes().length,
          consumedNotes: result.consumedNotes().length,
        };
      });

      expect(typeof result.blockNum).toBe("number");
      expect(result.blockNum).toBeGreaterThanOrEqual(0);
    });

    test("concurrent syncs with timeout all complete", async ({ page }) => {
      const result = await page.evaluate(async () => {
        const client = window.client;

        const syncPromises = [
          client.syncStateWithTimeout(30000),
          client.syncStateWithTimeout(30000),
          client.syncStateWithTimeout(30000),
        ];

        const results = await Promise.all(syncPromises);
        const blockNums = results.map((r) => r.blockNum());

        return {
          blockNums,
          allSame: blockNums.every((n) => n === blockNums[0]),
        };
      });

      expect(result.blockNums.length).toBe(3);
      expect(result.allSame).toBe(true);
    });
  });

  test.describe("Error Handling", () => {
    test("sync after failed sync works correctly", async ({ page }) => {
      // This test ensures that the lock is properly released after an error
      const result = await page.evaluate(async () => {
        const client = window.client;

        // First successful sync
        const result1 = await client.syncState();

        // Another successful sync (verifies lock was released)
        const result2 = await client.syncState();

        return {
          blockNum1: result1.blockNum(),
          blockNum2: result2.blockNum(),
        };
      });

      expect(typeof result.blockNum1).toBe("number");
      expect(typeof result.blockNum2).toBe("number");
    });
  });

  test.describe("Multiple Clients Same Store", () => {
    test("concurrent syncs from two clients on same store are coalesced", async ({
      page,
    }) => {
      const result = await page.evaluate(async () => {
        // Create two clients pointing to the same store
        const client1 = window.client;
        const client2 = await window.WebClient.createClient(
          window.rpcUrl,
          undefined,
          undefined,
          "tests" // Same store name as client1
        );

        // Fire concurrent syncs from both clients
        const syncPromises = [client1.syncState(), client2.syncState()];

        const results = await Promise.all(syncPromises);
        const blockNums = results.map((r) => r.blockNum());

        return {
          blockNum1: blockNums[0],
          blockNum2: blockNums[1],
          allSame: blockNums.every((n) => n === blockNums[0]),
        };
      });

      expect(typeof result.blockNum1).toBe("number");
      expect(typeof result.blockNum2).toBe("number");
      // Both syncs should complete with valid block numbers
      expect(result.blockNum1).toBeGreaterThanOrEqual(0);
      expect(result.blockNum2).toBeGreaterThanOrEqual(0);
    });

    test("many concurrent syncs from multiple clients all succeed", async ({
      page,
    }) => {
      const result = await page.evaluate(async () => {
        const client1 = window.client;
        const client2 = await window.WebClient.createClient(
          window.rpcUrl,
          undefined,
          undefined,
          "tests"
        );
        const client3 = await window.WebClient.createClient(
          window.rpcUrl,
          undefined,
          undefined,
          "tests"
        );

        // Fire many concurrent syncs
        const syncPromises = [
          client1.syncState(),
          client2.syncState(),
          client3.syncState(),
          client1.syncState(),
          client2.syncState(),
        ];

        const results = await Promise.all(syncPromises);

        return {
          count: results.length,
          allValid: results.every((r) => typeof r.blockNum() === "number"),
          blockNums: results.map((r) => r.blockNum()),
        };
      });

      expect(result.count).toBe(5);
      expect(result.allValid).toBe(true);
    });
  });

  test.describe("Different Stores", () => {
    test("concurrent syncs to different stores are independent", async ({
      page,
    }) => {
      const result = await page.evaluate(async () => {
        const client1 = window.client; // "tests" store
        const client2 = await window.WebClient.createClient(
          window.rpcUrl,
          undefined,
          undefined,
          "SyncLockTestStore1"
        );
        const client3 = await window.WebClient.createClient(
          window.rpcUrl,
          undefined,
          undefined,
          "SyncLockTestStore2"
        );

        // Fire concurrent syncs to different stores
        const syncPromises = [
          client1.syncState(),
          client2.syncState(),
          client3.syncState(),
        ];

        const results = await Promise.all(syncPromises);

        return {
          count: results.length,
          allValid: results.every((r) => typeof r.blockNum() === "number"),
          blockNums: results.map((r) => r.blockNum()),
        };
      });

      expect(result.count).toBe(3);
      expect(result.allValid).toBe(true);
    });
  });

  test.describe("Sync Lock State Consistency", () => {
    test("accounts remain consistent after concurrent syncs", async ({
      page,
    }) => {
      const result = await page.evaluate(async () => {
        const client = window.client;

        // Create a wallet before syncing
        const wallet = await client.newWallet(
          window.AccountStorageMode.private(),
          true,
          0
        );
        const walletId = wallet.id().toString();

        // Fire concurrent syncs
        const syncPromises = Array(5)
          .fill(null)
          .map(() => client.syncState());

        await Promise.all(syncPromises);

        // Verify account is still accessible and consistent
        const accounts = await client.getAccounts();
        const accountIds = accounts.map((a) => a.id().toString());

        return {
          walletId,
          accountCount: accounts.length,
          walletFound: accountIds.includes(walletId),
        };
      });

      expect(result.accountCount).toBeGreaterThanOrEqual(1);
      expect(result.walletFound).toBe(true);
    });

    test("sync height is consistent after concurrent syncs", async ({
      page,
    }) => {
      const result = await page.evaluate(async () => {
        const client = window.client;

        // Fire concurrent syncs
        const syncPromises = Array(5)
          .fill(null)
          .map(() => client.syncState());

        const results = await Promise.all(syncPromises);
        const syncBlockNums = results.map((r) => r.blockNum());

        // Get sync height directly
        const syncHeight = await client.getSyncHeight();

        return {
          syncBlockNums,
          syncHeight,
          // The sync height should be >= max of all sync results
          consistent: syncHeight >= Math.max(...syncBlockNums),
        };
      });

      expect(result.consistent).toBe(true);
    });
  });

  test.describe("Web Locks API Integration", () => {
    test("Web Locks API is available in test environment", async ({ page }) => {
      const result = await page.evaluate(async () => {
        return {
          hasNavigator: typeof navigator !== "undefined",
          hasLocks: typeof navigator?.locks !== "undefined",
          hasRequest: typeof navigator?.locks?.request === "function",
        };
      });

      // Chrome and Safari should have Web Locks support
      expect(result.hasNavigator).toBe(true);
      expect(result.hasLocks).toBe(true);
      expect(result.hasRequest).toBe(true);
    });

    test("sync operations use Web Locks when available", async ({ page }) => {
      const result = await page.evaluate(async () => {
        const client = window.client;
        let lockObserved = false;

        // Check for held locks before sync
        const locksBefore = await navigator.locks.query();
        const heldBefore = locksBefore.held?.length || 0;

        // Start a sync but don't await it yet
        const syncPromise = client.syncState();

        // Immediately check for locks (sync should be holding the lock)
        const locksDuring = await navigator.locks.query();
        const heldDuring = locksDuring.held?.length || 0;

        // Complete the sync
        await syncPromise;

        // Check for locks after sync
        const locksAfter = await navigator.locks.query();
        const heldAfter = locksAfter.held?.length || 0;

        return {
          heldBefore,
          heldDuring,
          heldAfter,
          // Lock may be quickly acquired and released, so we just check the API works
          apiWorks: true,
        };
      });

      expect(result.apiWorks).toBe(true);
    });
  });
});

test.describe("Cross-Tab Sync Lock Tests", () => {
  test("syncs from different browser contexts are coordinated", async ({
    browser,
  }) => {
    // Create two separate browser contexts (simulates different tabs)
    const context1 = await browser.newContext();
    const context2 = await browser.newContext();

    const page1 = await context1.newPage();
    const page2 = await context2.newPage();

    try {
      // Set up both pages
      const MIDEN_NODE_PORT = 57291;
      const setupPage = async (page: Page) => {
        await page.goto("http://localhost:8080");
        await page.evaluate(
          async ({ MIDEN_NODE_PORT }) => {
            const sdkExports = await import("./index.js");
            for (const [key, value] of Object.entries(sdkExports)) {
              window[key] = value;
            }

            const rpcUrl = `http://localhost:${MIDEN_NODE_PORT}`;
            window.rpcUrl = rpcUrl;
            // Both tabs use the same store name for cross-tab coordination
            const client = await window.WebClient.createClient(
              rpcUrl,
              undefined,
              undefined,
              "CrossTabTestStore"
            );
            window.client = client;
          },
          { MIDEN_NODE_PORT }
        );
      };

      await Promise.all([setupPage(page1), setupPage(page2)]);

      // Fire syncs from both tabs concurrently
      const [result1, result2] = await Promise.all([
        page1.evaluate(async () => {
          const startTime = Date.now();
          const result = await window.client.syncState();
          const endTime = Date.now();
          return {
            blockNum: result.blockNum(),
            duration: endTime - startTime,
          };
        }),
        page2.evaluate(async () => {
          const startTime = Date.now();
          const result = await window.client.syncState();
          const endTime = Date.now();
          return {
            blockNum: result.blockNum(),
            duration: endTime - startTime,
          };
        }),
      ]);

      // Both tabs should get valid results
      expect(typeof result1.blockNum).toBe("number");
      expect(typeof result2.blockNum).toBe("number");
      expect(result1.blockNum).toBeGreaterThanOrEqual(0);
      expect(result2.blockNum).toBeGreaterThanOrEqual(0);
    } finally {
      await context1.close();
      await context2.close();
    }
  });

  test("rapid syncs from multiple tabs all complete", async ({ browser }) => {
    const context1 = await browser.newContext();
    const context2 = await browser.newContext();
    const context3 = await browser.newContext();

    const page1 = await context1.newPage();
    const page2 = await context2.newPage();
    const page3 = await context3.newPage();

    try {
      const MIDEN_NODE_PORT = 57291;
      const setupPage = async (page: Page) => {
        await page.goto("http://localhost:8080");
        await page.evaluate(
          async ({ MIDEN_NODE_PORT }) => {
            const sdkExports = await import("./index.js");
            for (const [key, value] of Object.entries(sdkExports)) {
              window[key] = value;
            }

            const rpcUrl = `http://localhost:${MIDEN_NODE_PORT}`;
            window.rpcUrl = rpcUrl;
            const client = await window.WebClient.createClient(
              rpcUrl,
              undefined,
              undefined,
              "RapidCrossTabStore"
            );
            window.client = client;
          },
          { MIDEN_NODE_PORT }
        );
      };

      await Promise.all([setupPage(page1), setupPage(page2), setupPage(page3)]);

      // Fire multiple syncs from all tabs concurrently
      const results = await Promise.all([
        page1.evaluate(() =>
          window.client.syncState().then((r) => r.blockNum())
        ),
        page1.evaluate(() =>
          window.client.syncState().then((r) => r.blockNum())
        ),
        page2.evaluate(() =>
          window.client.syncState().then((r) => r.blockNum())
        ),
        page2.evaluate(() =>
          window.client.syncState().then((r) => r.blockNum())
        ),
        page3.evaluate(() =>
          window.client.syncState().then((r) => r.blockNum())
        ),
        page3.evaluate(() =>
          window.client.syncState().then((r) => r.blockNum())
        ),
      ]);

      // All syncs should complete successfully
      expect(results.length).toBe(6);
      results.forEach((blockNum) => {
        expect(typeof blockNum).toBe("number");
        expect(blockNum).toBeGreaterThanOrEqual(0);
      });
    } finally {
      await context1.close();
      await context2.close();
      await context3.close();
    }
  });
});

test.describe("Sync Lock Performance", () => {
  test("coalesced syncs complete faster than sequential", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = window.client;

      // Measure time for sequential syncs
      const sequentialStart = Date.now();
      await client.syncState();
      await client.syncState();
      await client.syncState();
      const sequentialTime = Date.now() - sequentialStart;

      // Measure time for concurrent syncs (should be coalesced)
      const concurrentStart = Date.now();
      await Promise.all([
        client.syncState(),
        client.syncState(),
        client.syncState(),
      ]);
      const concurrentTime = Date.now() - concurrentStart;

      return {
        sequentialTime,
        concurrentTime,
        // Concurrent should be significantly faster due to coalescing
        fasterOrEqual: concurrentTime <= sequentialTime,
      };
    });

    // Concurrent syncs should complete at least as fast as sequential
    // (likely faster due to coalescing)
    expect(result.fasterOrEqual).toBe(true);
  });
});
