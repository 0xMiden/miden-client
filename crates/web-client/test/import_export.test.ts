// Tests for import/export functionality using the mock chain (no running node required).

import test from "./playwright.global.setup";
import { expect } from "@playwright/test";

test.describe("export and import the db", () => {
  test("export db with an account, find the account when re-importing", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create();
      const walletId = wallet.id().toString();
      const commitment = wallet.to_commitment().toHex();

      // Export the store
      const storeData = await window.exportStore(client.storeIdentifier());

      // Create a new mock client and import the store
      const client2 = await window.MidenClient.createMock();
      await window.importStore(client2.storeIdentifier(), storeData);

      // Check the account exists in the new client
      const restored = await client2.accounts.get(walletId);
      return {
        restoredCommitment: restored?.to_commitment().toHex(),
        originalCommitment: commitment,
      };
    });

    expect(result.restoredCommitment).toEqual(result.originalCommitment);
  });
});

test.describe("export and import account", () => {
  test("should export and import a private account", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create({
        storage: "private",
        mutable: false,
        auth: "falcon",
      });
      const faucet = await client.accounts.create({
        type: window.AccountType.FungibleFaucet,
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      // Fund the wallet
      await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 1000n,
      });
      await client.sync();
      client.proveBlock();
      await client.sync();

      const walletId = wallet.id().toString();
      const faucetId = faucet.id().toString();

      const initialCommitment = (await client.accounts.get(walletId))
        ?.to_commitment()
        .toHex();
      const initialBalance = (await client.accounts.get(walletId))
        ?.vault()
        .getBalance(window.AccountId.fromHex(faucetId));

      // Export the account
      const accountFile = await client.accounts.export(wallet);
      const serialized = Array.from(accountFile.serialize());

      // Create a new mock client and import the account
      const client2 = await window.MidenClient.createMock();
      const bytes = new Uint8Array(serialized);
      const deserialized = window.AccountFile.deserialize(bytes);
      await client2.accounts.import({ file: deserialized });

      const restored = await client2.accounts.get(walletId);
      const restoredCommitment = restored?.to_commitment().toHex();
      const restoredBalance = restored
        ?.vault()
        .getBalance(window.AccountId.fromHex(faucetId));

      return {
        initialCommitment,
        restoredCommitment,
        initialBalance: initialBalance?.toString(),
        restoredBalance: restoredBalance?.toString(),
      };
    });

    expect(result.restoredCommitment).toEqual(result.initialCommitment);
    expect(result.restoredBalance).toEqual(result.initialBalance);
  });
});

test.describe("export and import note", () => {
  const exportTypes = [
    ["Id", "NoteId"],
    ["Details", "NoteDetails"],
  ];

  exportTypes.forEach(([exportType, expectedNoteType]) => {
    test(`export note as note file -- export type: ${exportType}`, async ({
      page,
    }) => {
      const result = await page.evaluate(
        async ({ exportType }) => {
          const client = await window.MidenClient.createMock();
          const wallet = await client.accounts.create();
          const faucet = await client.accounts.create({
            type: window.AccountType.FungibleFaucet,
            symbol: "DAG",
            decimals: 8,
            maxSupply: 10_000_000n,
          });

          await client.transactions.mint({
            account: faucet,
            to: wallet,
            amount: 500n,
            type: "public",
          });
          await client.sync();
          client.proveBlock();
          await client.sync();

          const notes = await client.notes.list();
          const noteId = notes[0].id().toString();

          const format =
            window.NoteExportFormat[
              exportType as keyof typeof window.NoteExportFormat
            ];
          const noteFile = await client.notes.export(noteId, { format });
          return noteFile.noteType();
        },
        { exportType }
      );

      expect(result).toBe(expectedNoteType);
    });
  });

  test(`exporting non-existing note fails`, async ({ page }) => {
    await page.evaluate(async () => {
      await window.MidenClient.createMock();
    });

    // Random note id
    const noteId =
      "0x60b06dbb6c7435ab1d439df972e483bca43bc21654dce2611de98ec3896beaed";
    await expect(
      page.evaluate(
        async ({ noteId }) => {
          const client = await window.MidenClient.createMock();
          const format = window.NoteExportFormat.Details;
          return await client.notes.export(noteId, { format });
        },
        { noteId }
      )
    ).rejects.toThrowError("No output note found");
  });

  test(`exporting and then importing note`, async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create();
      const faucet = await client.accounts.create({
        type: window.AccountType.FungibleFaucet,
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 500n,
        type: "public",
      });
      await client.sync();
      client.proveBlock();
      await client.sync();

      const notes = await client.notes.list();
      const noteId = notes[0].id().toString();

      // Export the note with Details format
      const format = window.NoteExportFormat.Details;
      const noteFile = await client.notes.export(noteId, { format });
      const serialized = noteFile.serialize();

      // Create a new mock client and import the note
      const client2 = await window.MidenClient.createMock();
      const deserialized = window.NoteFile.deserialize(serialized);
      const importedNoteId = await client2.notes.import(deserialized);

      return {
        originalNoteId: noteId,
        importedNoteId: importedNoteId.toString(),
      };
    });

    expect(result.importedNoteId).toBe(result.originalNoteId);
  });

  test(`export output note`, async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const account1 = await client.accounts.create();
      const account2 = await client.accounts.create();

      const p2IdNote = window.Note.createP2IDNote(
        account1.id(),
        account2.id(),
        new window.NoteAssets([]),
        window.NoteType.Public,
        new window.NoteAttachment()
      );
      return window.NoteFile.fromRawOutputNote(
        window.RawOutputNote.full(p2IdNote)
      ).noteType();
    });

    expect(result).toBe("NoteDetails");
  });

  test(`export input note`, async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create();
      const faucet = await client.accounts.create({
        type: window.AccountType.FungibleFaucet,
        symbol: "DAG",
        decimals: 8,
        maxSupply: 10_000_000n,
      });

      // Mint
      const { txId: mintTxId } = await client.transactions.mint({
        account: faucet,
        to: wallet,
        amount: 500n,
        type: "public",
      });
      await client.sync();
      client.proveBlock();
      await client.sync();

      // Get the minted note ID
      const txRecords = await client.transactions.list({
        ids: [mintTxId.toHex()],
      });
      const mintedNoteId = txRecords[0]
        .rawOutputNotes()
        .notes()[0]
        .id()
        .toString();

      // Consume the note so it becomes an input note
      await client.transactions.consume({
        account: wallet,
        notes: mintedNoteId,
      });
      client.proveBlock();
      await client.sync();

      // Get the input note and export
      const inputNoteRecord = await client.notes.get(mintedNoteId);
      return window.NoteFile.fromInputNote(
        inputNoteRecord.toInputNote()
      ).noteType();
    });

    expect(result).toBe("NoteWithProof");
  });
});
