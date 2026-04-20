// @ts-nocheck
// Guards against a regression where `dist/eager.js` silently stops being eager
// (e.g. a future Rollup tweak accidentally strips the TLA, matching the same
// kind of miss that #2010 was addressing on the lazy side).
//
// The contract of the eager entry is: importing it is the ONLY await the caller
// should need before touching wasm-bindgen types. If that contract breaks, the
// sync-constructor assertions below throw
// `TypeError: Cannot read properties of undefined (reading '<wasm fn>')`.
//
// The paired lazy assertion is the guardrail in the other direction: if
// `dist/index.js` ever accidentally starts TLA-ing WASM at the top level, the
// two entry points will have silently converged and we'd lose the
// Capacitor/SSR-safe path. That one is expected to throw.
import { test as base, expect } from "@playwright/test";

base.beforeEach(async ({ page }) => {
  page.on("pageerror", (err) => {
    console.error("PAGE ERROR:", err);
  });
  await page.goto("http://localhost:8080");
});

base(
  "dist/eager.js: wasm-bindgen constructors work synchronously after import",
  async ({ page }) => {
    const result = await page.evaluate(async () => {
      // The dynamic import below is the ONLY await. If eager.js has a working
      // top-level `await getWasmOrThrow()`, the import promise resolves with
      // WASM already initialized and every call on the next lines is sync.
      const mod = await import("./eager.js");

      // Bare constructor — wasm.felt_new is called synchronously.
      const felt = new mod.Felt(42n);

      // Static factory with a valid input — wasm.transactionprover_newLocalProver
      // is called synchronously. This one is specifically relevant because the
      // surrounding work on this branch is about when the prover's WASM is
      // ready.
      const prover = mod.TransactionProver.newLocalProver();

      return {
        feltAsInt: felt.asInt().toString(),
        proverTypeName: prover.constructor.name,
      };
    });

    expect(result.feltAsInt).toBe("42");
    expect(result.proverTypeName).toBe("TransactionProver");
  }
);

base(
  "dist/index.js: sync wasm-bindgen constructor throws before getWasmOrThrow resolves",
  async ({ page }) => {
    const result = await page.evaluate(async () => {
      const mod = await import("./index.js");

      // No `await mod.getWasmOrThrow()` here on purpose. The lazy entry must
      // NOT initialize WASM at import time; if it ever does, this assertion
      // flips and we've silently merged the two entry points.
      let threw = false;
      let message = "";
      try {
        new mod.Felt(1n);
      } catch (e) {
        threw = true;
        message = e instanceof Error ? e.message : String(e);
      }

      return { threw, message };
    });

    expect(result.threw).toBe(true);
    // The exact message is "Cannot read properties of undefined (reading
    // 'felt_new')" in V8; WebKit wraps the same failure as "undefined is not
    // an object (evaluating 'wasm.felt_new')". Both reference the missing
    // wasm-bindgen export, which is the signal we want.
    expect(result.message.toLowerCase()).toContain("felt_new");
  }
);
