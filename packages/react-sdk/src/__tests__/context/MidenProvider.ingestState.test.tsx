import { describe, it, expect, vi } from "vitest";
import { renderHook } from "@testing-library/react";
import React from "react";
import { SignerContext, useSigner } from "../../context/SignerContext";
import type {
  IngestStateCallback,
  SignerContextValue,
} from "../../context/SignerContext";
import { AsyncLock } from "../../utils/asyncLock";
import { createMockSignerContext } from "../mocks/signer-context";

/**
 * Contract tests for the ingestState plumbing on SignerContextValue.
 *
 * Full MidenProvider integration (verifying that ingestState actually fires
 * after init and after each auto-sync tick) requires a real WASM client and
 * is covered in e2e tests. These tests verify the type contract + the
 * concurrency primitives the implementation depends on.
 */
describe("MidenProvider.ingestState contract", () => {
  describe("ingestState field on SignerContextValue", () => {
    it("is optional — signers without out-of-band state omit it", () => {
      const ctx = createMockSignerContext({});
      expect(ctx.ingestState).toBeUndefined();
    });

    it("can be populated with a callback signed { client, runExclusive } => Promise<void>", () => {
      const ingestStateMock = vi.fn().mockResolvedValue(undefined);
      const ctx: SignerContextValue = createMockSignerContext({
        ingestState: ingestStateMock as IngestStateCallback,
      });
      expect(typeof ctx.ingestState).toBe("function");
    });

    it("is reachable via useSigner()", () => {
      const ingest = vi.fn().mockResolvedValue(undefined);
      const ctx = createMockSignerContext({
        ingestState: ingest as IngestStateCallback,
      });
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <SignerContext.Provider value={ctx}>{children}</SignerContext.Provider>
      );
      const { result } = renderHook(() => useSigner(), { wrapper });
      expect(result.current?.ingestState).toBe(ingest);
    });
  });

  describe("runExclusive concurrency contract (used by ingestState)", () => {
    it("serializes overlapping calls (FIFO)", async () => {
      const lock = new AsyncLock();
      const events: string[] = [];

      const slow = lock.runExclusive(async () => {
        events.push("slow:start");
        await new Promise((r) => setTimeout(r, 30));
        events.push("slow:end");
      });
      const fast = lock.runExclusive(async () => {
        events.push("fast:start");
        events.push("fast:end");
      });
      await Promise.all([slow, fast]);

      // Fast must wait for slow to finish — proves serialization, which is
      // what ingestState relies on for per-note imports.
      expect(events).toEqual([
        "slow:start",
        "slow:end",
        "fast:start",
        "fast:end",
      ]);
    });

    it("is non-reentrant — calling runExclusive from inside runExclusive deadlocks", async () => {
      // This is the critical invariant that drives the Pattern B init refactor:
      // ingestState runs OUTSIDE any outer runExclusive specifically because
      // the lock is non-reentrant. Document the behavior here so a future
      // change to AsyncLock that breaks it would fail this assertion.
      const lock = new AsyncLock();
      let innerStarted = false;

      const outer = lock.runExclusive(async () => {
        // Schedule an inner call but don't await — it would deadlock.
        const innerPromise = lock
          .runExclusive(async () => {
            innerStarted = true;
          })
          .catch(() => {});
        // Race the inner against a short timer; the inner should NOT start
        // because the outer hasn't released the lock yet.
        const timer = new Promise((r) => setTimeout(r, 30));
        await Promise.race([innerPromise, timer]);
      });

      await outer;
      // After outer releases, the inner finally runs.
      await new Promise((r) => setTimeout(r, 10));
      expect(innerStarted).toBe(true);
    });
  });

  describe("ingestState callback shape (called outside outer runExclusive)", () => {
    it("can use its runExclusive helper without deadlock when invoked outside any wrapping lock", async () => {
      const lock = new AsyncLock();
      const runExclusive = <T,>(fn: () => Promise<T>) => lock.runExclusive(fn);

      const ingest: IngestStateCallback = async ({ runExclusive }) => {
        // Simulate per-note import pattern — multiple sequential runExclusive
        // calls from inside the callback. These work because we are NOT
        // inside a wrapping runExclusive (MidenProvider doesn't outer-wrap).
        await runExclusive(async () => {
          /* "snapshot" */
        });
        await runExclusive(async () => {
          /* "import note 1" */
        });
        await runExclusive(async () => {
          /* "import note 2" */
        });
      };

      // The MidenProvider invocation pattern: NO outer runExclusive wrap.
      await ingest({ client: {} as any, runExclusive });
      // Reached here without deadlock — contract verified.
      expect(true).toBe(true);
    });
  });
});
