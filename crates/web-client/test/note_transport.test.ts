import test from "./playwright.global.setup";
import { Page, expect } from "@playwright/test";

test("transport basic", async ({ page }) => {
  const result = await page.evaluate(async () => {
    const client = await window._MockWebClient.createClient();

    // Create 32-byte seeds
    const senderSeed = new Uint8Array(32).fill(1);
    const recipientSeed = new Uint8Array(32).fill(2);

    // Create accounts on the same client
    const senderAccount = await client.newWallet(
      window.AccountStorageMode.private(),
      true,
      0,
      senderSeed
    );
    const recipientAccount = await client.newWallet(
      window.AccountStorageMode.private(),
      true,
      0,
      recipientSeed
    );

    // Create recipient address
    const recipientAddress = window.Address.fromAccountId(
      recipientAccount.id(),
      "BasicWallet"
    );

    // Create note
    const noteAssets = new window.NoteAssets([]);
    const note = window.Note.createP2IDNote(
      senderAccount.id(),
      recipientAccount.id(),
      noteAssets,
      window.NoteType.Private,
      new window.NoteAttachment()
    );

    // Sync-state / fetch notes
    // No notes before sending
    await client.fetchPrivateNotes();
    let notes = await client.getInputNotes(
      new window.NoteFilter(window.NoteFilterTypes.All)
    );
    const notesBeforeSending = notes.length;

    // Send note
    await client.sendPrivateNote(note, recipientAddress);

    // Sync-state / fetch notes
    // 1 note stored
    await client.fetchPrivateNotes();
    notes = await client.getInputNotes(
      new window.NoteFilter(window.NoteFilterTypes.All)
    );
    const notesAfterSending = notes.length;

    // Sync again, should be only 1 note stored
    await client.fetchPrivateNotes();
    notes = await client.getInputNotes(
      new window.NoteFilter(window.NoteFilterTypes.All)
    );
    const notesAfterSecondSync = notes.length;

    return {
      notesBeforeSending,
      notesAfterSending,
      notesAfterSecondSync,
    };
  });

  // Assertions in the main test context
  expect(result.notesBeforeSending).toBe(0);
  expect(result.notesAfterSending).toBe(1);
  expect(result.notesAfterSecondSync).toBe(1);
});
