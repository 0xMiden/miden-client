// @ts-nocheck
import { expect } from "@playwright/test";
import test, { getRpcUrl, RUN_ID } from "./playwright.global.setup";

/**
 * Sync performance benchmarks.
 *
 * These tests measure the time it takes to sync from genesis against a node
 * that has been seeded with data. They are designed to be run manually for
 * A/B comparisons between branches rather than as part of the regular CI suite.
 *
 * Usage:
 *   yarn playwright test -g "Sync Benchmark" --project chromium
 *
 * Environment:
 *   TEST_MIDEN_RPC_URL or TEST_MIDEN_NETWORK to point at the seeded node.
 *   BENCH_ITERATIONS (default: 3) — number of fresh-client sync runs.
 */

const ITERATIONS = parseInt(process.env.BENCH_ITERATIONS ?? "3", 10);
const BENCH_ACCOUNT_ID = "0x742674d972ee450072c628defb488e";

test.describe("Sync Benchmark", () => {
  // Increase timeout — full syncs against large stores can take a while.
  test.setTimeout(0);

  test("full sync from genesis", async ({ page }) => {
    const rpcUrl = getRpcUrl();
    page.on("console", (msg) => console.log(`[browser] ${msg.text()}`));

    const result = await page.evaluate(
      async ({ rpcUrl, runId, iterations, accountIdHex }) => {
        const durations: number[] = [];
        const summaries: {
          blockNum: number;
          committedNotes: number;
          consumedNotes: number;
          updatedAccounts: number;
          committedTransactions: number;
        }[] = [];

        for (let i = 0; i < iterations; i++) {
          // Create a fresh client with a unique store so each run starts from
          // block 0, giving us a full sync measurement.
          const storeName = `bench_${runId}_iter${i}`;
          console.log(`[iter ${i}] creating client with store ${storeName}...`);
          const client = await window.WasmWebClient.createClient(
            rpcUrl,
            undefined,
            undefined,
            storeName
          );
          console.log(`[iter ${i}] client created, importing account ${accountIdHex}...`);

          // Import the account that consumed the seeded notes.
          const accountId = window.AccountId.fromHex(accountIdHex);
          await client.importAccountById(accountId);
          console.log(`[iter ${i}] account imported, starting sync...`);

          // Sync to tip, measuring wall-clock time.
          const start = performance.now();

          // Loop until we reach the chain tip (syncState advances in batches).
          let summary;
          let prevBlock = -1;
          let steps = 0;
          let totalCommitted = 0;
          let totalConsumed = 0;
          while (true) {
            summary = await client.syncState();
            const currentBlock = summary.blockNum();
            const committed = summary.committedNotes().length;
            const consumed = summary.consumedNotes().length;
            totalCommitted += committed;
            totalConsumed += consumed;
            steps++;
            console.log(
              `[iter ${i}] step ${steps}: block ${currentBlock}, ` +
              `+${committed} committed, +${consumed} consumed, ` +
              `elapsed ${((performance.now() - start) / 1000).toFixed(1)}s`
            );
            if (currentBlock === prevBlock) break;
            prevBlock = currentBlock;
          }

          const elapsed = performance.now() - start;
          durations.push(elapsed);

          summaries.push({
            blockNum: summary.blockNum(),
            committedNotes: totalCommitted,
            consumedNotes: totalConsumed,
            updatedAccounts: summary.updatedAccounts().length,
            committedTransactions: summary.committedTransactions().length,
          });
        }

        // Compute stats.
        const sorted = [...durations].sort((a, b) => a - b);
        const mean = durations.reduce((s, v) => s + v, 0) / durations.length;
        const median = sorted[Math.floor(sorted.length / 2)];
        const min = sorted[0];
        const max = sorted[sorted.length - 1];

        return { durations, summaries, stats: { mean, median, min, max } };
      },
      { rpcUrl, runId: RUN_ID, iterations: ITERATIONS, accountIdHex: BENCH_ACCOUNT_ID }
    );

    // Print results so they're visible in the test output.
    console.log("\n=== Sync Benchmark Results ===");
    console.log(`Iterations: ${ITERATIONS}`);
    console.log(`RPC URL:    ${rpcUrl}`);
    console.log(`Account:    ${BENCH_ACCOUNT_ID}`);
    console.log("-----------------------------");
    for (let i = 0; i < result.durations.length; i++) {
      const s = result.summaries[i];
      console.log(
        `  Run ${i + 1}: ${result.durations[i].toFixed(0)}ms ` +
          `(block ${s.blockNum}, committed ${s.committedNotes} notes, ` +
          `consumed ${s.consumedNotes} notes, ` +
          `${s.updatedAccounts} accounts, ` +
          `${s.committedTransactions} txns)`
      );
    }
    console.log("-----------------------------");
    console.log(`  Mean:   ${result.stats.mean.toFixed(0)}ms`);
    console.log(`  Median: ${result.stats.median.toFixed(0)}ms`);
    console.log(`  Min:    ${result.stats.min.toFixed(0)}ms`);
    console.log(`  Max:    ${result.stats.max.toFixed(0)}ms`);
    console.log("=============================\n");

    // Sanity checks — the sync should have progressed.
    expect(result.summaries[0].blockNum).toBeGreaterThan(0);
    expect(result.durations.length).toBe(ITERATIONS);
  });

  test("incremental sync (already at tip)", async ({ page }) => {
    const rpcUrl = getRpcUrl();
    page.on("console", (msg) => console.log(`[browser] ${msg.text()}`));

    const result = await page.evaluate(
      async ({ rpcUrl, runId, iterations, accountIdHex }) => {
        // Create client and import the account.
        const storeName = `bench_incr_${runId}`;
        const client = await window.WasmWebClient.createClient(
          rpcUrl,
          undefined,
          undefined,
          storeName
        );

        const accountId = window.AccountId.fromHex(accountIdHex);
        await client.importAccountById(accountId);

        // Full sync to reach the tip.
        let prev = -1;
        let steps = 0;
        const warmupStart = performance.now();
        while (true) {
          const s = await client.syncState();
          const cur = s.blockNum();
          steps++;
          console.log(
            `[warmup] step ${steps}: block ${cur}, ` +
            `elapsed ${((performance.now() - warmupStart) / 1000).toFixed(1)}s`
          );
          if (cur === prev) break;
          prev = cur;
        }

        const tipBlock = prev;

        // Now measure incremental syncs (no new data expected).
        const durations: number[] = [];
        for (let i = 0; i < iterations; i++) {
          const start = performance.now();
          await client.syncState();
          durations.push(performance.now() - start);
        }

        const sorted = [...durations].sort((a, b) => a - b);
        const mean = durations.reduce((s, v) => s + v, 0) / durations.length;
        const median = sorted[Math.floor(sorted.length / 2)];

        return {
          tipBlock,
          durations,
          stats: { mean, median, min: sorted[0], max: sorted[sorted.length - 1] },
        };
      },
      { rpcUrl, runId: RUN_ID, iterations: ITERATIONS, accountIdHex: BENCH_ACCOUNT_ID }
    );

    console.log("\n=== Incremental Sync Benchmark ===");
    console.log(`Tip block: ${result.tipBlock}`);
    console.log(`Iterations: ${ITERATIONS}`);
    console.log("----------------------------------");
    for (let i = 0; i < result.durations.length; i++) {
      console.log(`  Run ${i + 1}: ${result.durations[i].toFixed(1)}ms`);
    }
    console.log("----------------------------------");
    console.log(`  Mean:   ${result.stats.mean.toFixed(1)}ms`);
    console.log(`  Median: ${result.stats.median.toFixed(1)}ms`);
    console.log(`  Min:    ${result.stats.min.toFixed(1)}ms`);
    console.log(`  Max:    ${result.stats.max.toFixed(1)}ms`);
    console.log("==================================\n");

    expect(result.tipBlock).toBeGreaterThan(0);
  });
});
