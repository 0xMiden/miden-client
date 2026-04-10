// @ts-nocheck
import { test, expect } from "./test-setup";

test("transport basic", async ({ run }) => {
  const result = await run(async ({ client, sdk, helpers }) => {
    const mockClient = await helpers.createFreshMockClient();

    // Create 32-byte seeds
    const senderSeed = new Uint8Array(32).fill(1);
    const recipientSeed = new Uint8Array(32).fill(2);

    // Create accounts on the same client
    const senderAccount = await mockClient.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512,
      senderSeed
    );
    const recipientAccount = await mockClient.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512,
      recipientSeed
    );

    // Create recipient address
    const recipientAddress = sdk.Address.fromAccountId(
      recipientAccount.id(),
      "BasicWallet"
    );

    // Create note
    const noteAssets = new sdk.NoteAssets([]);
    const note = sdk.Note.createP2IDNote(
      senderAccount.id(),
      recipientAccount.id(),
      noteAssets,
      sdk.NoteType.Private,
      new sdk.NoteAttachment()
    );

    // No notes before sending
    await mockClient.fetchPrivateNotes();
    let notes = await mockClient.getInputNotes(
      new sdk.NoteFilter(sdk.NoteFilterTypes.All)
    );
    const notesBeforeSending = notes.length;

    // Send note
    await mockClient.sendPrivateNote(note, recipientAddress);

    // 1 note stored
    await mockClient.fetchPrivateNotes();
    notes = await mockClient.getInputNotes(
      new sdk.NoteFilter(sdk.NoteFilterTypes.All)
    );
    const notesAfterSending = notes.length;

    // Sync again, should be only 1 note stored
    await mockClient.fetchPrivateNotes();
    notes = await mockClient.getInputNotes(
      new sdk.NoteFilter(sdk.NoteFilterTypes.All)
    );
    const notesAfterSecondSync = notes.length;

    return { notesBeforeSending, notesAfterSending, notesAfterSecondSync };
  });

  expect(result.notesBeforeSending).toBe(0);
  expect(result.notesAfterSending).toBe(1);
  expect(result.notesAfterSecondSync).toBe(1);
});
