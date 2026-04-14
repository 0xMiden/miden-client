// @ts-nocheck
import { mockTest as test } from "./playwright.global.setup";
import { expect } from "@playwright/test";

/**
 * Exercises the idxdb store's sync path over a chain long enough to cross MMR
 * peak-layout boundaries. Includes "red herring" notes — notes whose tags the
 * client tracks but that are addressed to a fake wallet — exercising the
 * `found_relevant_note = false` path in state_sync.
 */

const BOUNDARIES = [256, 512, 1024, 2048];
const NUM_RED_HERRINGS = 5;

const runCatchUpSync = async (page, gap) => {
  return await page.evaluate(async ([gap, numHerrings]) => {
    const client = await window.MockWasmWebClient.createClient();
    await client.syncState();

    const wallet = await client.newWallet(
      window.AccountStorageMode.private(),
      true,
      window.AuthScheme.AuthRpoFalcon512
    );
    const faucet = await client.newFaucet(
      window.AccountStorageMode.private(),
      false,
      "TST",
      8,
      BigInt(10000000),
      window.AuthScheme.AuthRpoFalcon512
    );

    // Red herring setup: create a fake AccountId that the client does NOT own.
    // Track its tag so sync_notes returns blocks with notes to it, but the
    // screener discards them (found_relevant_note = false).
    const wasm = await window.getWasmOrThrow();
    const fakeAccountId = wasm.AccountId.fromHex(
      "0x0032000000000001ffffffffffffffff"
    );
    const fakeTag = wasm.Address.fromAccountId(fakeAccountId, undefined)
      .toNoteTag()
      .asU32()
      .toString();
    await client.addTag(fakeTag);

    // Mint real note to the main wallet.
    const mintReq = await client.newMintTransactionRequest(
      wallet.id(),
      faucet.id(),
      window.NoteType.Public,
      BigInt(1000)
    );
    await client.submitNewTransaction(faucet.id(), mintReq);
    client.proveBlock();

    // Scatter red herring notes throughout the chain — each in its own block.
    // These are minted to the fake account; the client tracks the tag but
    // doesn't own the account, so the screener discards the notes.
    for (let i = 0; i < numHerrings; i++) {
      const herringReq = await client.newMintTransactionRequest(
        fakeAccountId,
        faucet.id(),
        window.NoteType.Public,
        BigInt(100)
      );
      await client.submitNewTransaction(faucet.id(), herringReq);
      client.proveBlock();

      const portion = Math.floor(gap / (numHerrings + 1));
      if (portion > 0) {
        client.advanceBlocks(portion);
      }
    }

    // Advance remaining blocks.
    const usedBlocks = Math.floor(gap / (numHerrings + 1)) * numHerrings;
    const remaining = gap - usedBlocks;
    if (remaining > 0) {
      client.advanceBlocks(remaining);
    }

    // Catch-up sync across the whole delta: real note + red herrings + gap.
    await client.syncState();

    // Consume only the real note.
    const mintedNotes = await client.getInputNotes(
      new window.NoteFilter(window.NoteFilterTypes.Committed)
    );
    for (const noteRecord of mintedNotes) {
      try {
        const note = noteRecord.toNote();
        const consumeReq = client.newConsumeTransactionRequest([note]);
        await client.submitNewTransaction(wallet.id(), consumeReq);
        client.proveBlock();
        await client.syncState();
        break;
      } catch (e) {
        // P2ID target mismatch — red herring note, skip.
        continue;
      }
    }

    const account = await client.getAccount(wallet.id());
    const balance = account.vault().getBalance(faucet.id()).toString();

    client.free();
    return balance;
  }, [gap, NUM_RED_HERRINGS]);
};

test.describe("sync over long chain (idxdb)", () => {
  test.describe.configure({ timeout: 720000 });

  for (const boundary of BOUNDARIES) {
    for (const offset of [-1, 0, 1]) {
      const gap = boundary + offset;

      test(`catch-up sync across ${gap}-block gap`, async ({ page }) => {
        const balance = await runCatchUpSync(page, gap);
        expect(balance).toEqual("1000");
      });
    }
  }
});
