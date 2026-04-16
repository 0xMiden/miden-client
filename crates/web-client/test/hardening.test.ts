//@ts-nocheck
//
// SDK hardening regression tests.
//
// Exercises the primitives added to protect against the failure modes we
// uncovered in the wallet's 1000-op stress run and the SDK review that
// followed:
//
//   * `_serializeWasmCall`  — WebClient serializes concurrent mutating
//     calls so wasm-bindgen's "recursive use of an object detected"
//     panic can't fire, and so a caller doesn't have to maintain its
//     own mutex (the wallet used to).
//   * `waitForIdle`         — lets a caller wait for the in-flight
//     serialized call to settle before performing a non-WASM action
//     (e.g. clearing an in-memory vault key on lock).
//   * `lastAuthError`       — captures the raw JS value a sign
//     callback threw so consumers can recover structured metadata
//     (e.g. `reason: 'locked'`) that the kernel-level auth::request
//     diagnostic would otherwise erase.
//   * `resendPrivateNoteById` — re-sends a previously-sent private
//     note via the transport layer, looking it up from the local
//     output-note store. Needed to recover from transport failures
//     where the on-chain commit landed but the recipient never got
//     the note blob.
//
// The tests run against a MockWasmWebClient — they don't hit a real
// node, so they're safe to run in CI without network setup.

import { mockTest as test } from "./playwright.global.setup";
import { expect } from "@playwright/test";

test.describe("WebClient hardening", () => {
  test.describe("serialized mutating calls", () => {
    test("concurrent newWallet calls all succeed without panic", async ({
      page,
    }) => {
      // 5 concurrent account creations exercise `_serializeWasmCall` on
      // the direct-path newWallet method. Without serialization this
      // trips "recursive use of an object detected" inside the wasm
      // binding (wasm-bindgen's internal RefCell).
      const result = await page.evaluate(async () => {
        const client = await window.MockWasmWebClient.createClient();

        const seeds = Array.from({ length: 5 }, (_, i) =>
          new Uint8Array(32).fill(i + 1)
        );
        const promises = seeds.map((seed) =>
          client.newWallet(
            window.AccountStorageMode.private(),
            true,
            window.AuthScheme.AuthRpoFalcon512,
            seed
          )
        );
        const accounts = await Promise.all(promises);
        const ids = accounts.map((a) => a.id().toString());
        return {
          created: ids.length,
          unique: new Set(ids).size,
        };
      });

      expect(result.created).toBe(5);
      expect(result.unique).toBe(5);
    });

    test("waitForIdle resolves after in-flight call completes", async ({
      page,
    }) => {
      // Kick off a long-ish WASM call (newWallet) without awaiting,
      // then await waitForIdle. The second promise must resolve AFTER
      // the first — otherwise the chain semantics are broken.
      const result = await page.evaluate(async () => {
        const client = await window.MockWasmWebClient.createClient();

        const events: string[] = [];
        const seed = new Uint8Array(32).fill(42);

        const walletPromise = client
          .newWallet(
            window.AccountStorageMode.private(),
            true,
            window.AuthScheme.AuthRpoFalcon512,
            seed
          )
          .then(() => events.push("newWallet"));

        // Not awaited before the next call — waitForIdle should wait
        // for the queued newWallet above.
        const idlePromise = client.waitForIdle().then(() => events.push("idle"));

        await Promise.all([walletPromise, idlePromise]);
        return events;
      });

      // waitForIdle MUST NOT resolve before newWallet.
      expect(result[0]).toBe("newWallet");
      expect(result[1]).toBe("idle");
    });

    test("waitForIdle on an empty chain resolves immediately", async ({
      page,
    }) => {
      // With no queued call, waitForIdle should be a cheap no-op.
      const elapsedMs = await page.evaluate(async () => {
        const client = await window.MockWasmWebClient.createClient();
        const t0 = performance.now();
        await client.waitForIdle();
        return performance.now() - t0;
      });

      // Generous upper bound — we only care that it doesn't hang.
      expect(elapsedMs).toBeLessThan(100);
    });
  });

  test.describe("lastAuthError", () => {
    test("returns null when no sign call has happened", async ({ page }) => {
      const result = await page.evaluate(async () => {
        const client = await window.MockWasmWebClient.createClient();
        return client.lastAuthError();
      });
      expect(result).toBeNull();
    });
  });

  test.describe("resendPrivateNoteById", () => {
    test("re-sends a previously-sent output note", async ({ page }) => {
      // The mock transport records notes it receives. Send a note
      // once, then resend by ID — the transport should receive the
      // same note twice (the recipient's fetch would get the delivery
      // either way; we're checking the lookup + reconstruction path).
      const result = await page.evaluate(async () => {
        const client = await window.MockWasmWebClient.createClient();

        const senderSeed = new Uint8Array(32).fill(1);
        const recipientSeed = new Uint8Array(32).fill(2);

        const sender = await client.newWallet(
          window.AccountStorageMode.private(),
          true,
          window.AuthScheme.AuthRpoFalcon512,
          senderSeed
        );
        const recipient = await client.newWallet(
          window.AccountStorageMode.private(),
          true,
          window.AuthScheme.AuthRpoFalcon512,
          recipientSeed
        );

        const recipientAddress = window.Address.fromAccountId(
          recipient.id(),
          "BasicWallet"
        );

        const note = window.Note.createP2IDNote(
          sender.id(),
          recipient.id(),
          new window.NoteAssets([]),
          window.NoteType.Private,
          new window.NoteAttachment()
        );
        const noteId = note.id();

        // First send to populate transport.
        await client.sendPrivateNote(note, recipientAddress);

        // Resend by ID — this is the hardening path: no local Note
        // handle needed, the SDK looks it up.
        await client.resendPrivateNoteById(noteId, recipientAddress);

        // Recipient fetches — should see the note (one copy is enough,
        // transport dedups by note ID, but the point is resend didn't
        // throw).
        await client.fetchPrivateNotes();
        const notes = await client.getInputNotes(
          new window.NoteFilter(window.NoteFilterTypes.All)
        );
        return {
          noteCount: notes.length,
          firstNoteId: notes[0]?.id().toString(),
          expectedId: noteId.toString(),
        };
      });

      expect(result.noteCount).toBe(1);
      expect(result.firstNoteId).toBe(result.expectedId);
    });

    test("throws when note-id is not tracked locally", async ({ page }) => {
      // If the caller passes a note-id the client doesn't have in its
      // output-note store, the SDK surfaces a clear error rather than
      // silently no-opping.
      const error = await page.evaluate(async () => {
        const client = await window.MockWasmWebClient.createClient();

        const seed = new Uint8Array(32).fill(7);
        const acct = await client.newWallet(
          window.AccountStorageMode.private(),
          true,
          window.AuthScheme.AuthRpoFalcon512,
          seed
        );
        const addr = window.Address.fromAccountId(acct.id(), "BasicWallet");

        // Fabricate a note we never actually sent.
        const bogus = window.Note.createP2IDNote(
          acct.id(),
          acct.id(),
          new window.NoteAssets([]),
          window.NoteType.Private,
          new window.NoteAttachment()
        );
        try {
          await client.resendPrivateNoteById(bogus.id(), addr);
          return { threw: false };
        } catch (e) {
          return {
            threw: true,
            message: String((e as Error).message || e),
          };
        }
      });

      expect(error.threw).toBe(true);
      // Message should name the missing note — don't lock onto exact
      // wording, just confirm the error surfaces.
      expect(error.message.toLowerCase()).toContain("note");
    });
  });
});
