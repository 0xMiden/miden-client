// Tests for import/export functionality using the mock chain (no running node required).

import test from "./playwright.global.setup";
import { expect } from "@playwright/test";
import { clearStore } from "./webClientTestUtils";

test.describe("export and import the db", () => {
  test("export db with an account, find the account when re-importing", async ({
    page,
  }) => {
    const exported = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const wallet = await client.accounts.create();
      const walletId = wallet.id().toString();
      const storeData = await window.exportStore(client.storeIdentifier());
      return {
        walletId,
        commitment: wallet.to_commitment().toHex(),
        storeName: client.storeIdentifier(),
        storeData,
      };
    });

    await clearStore(page, exported.storeName);

    const restoredCommitment = await page.evaluate(async ({ storeData, walletId }) => {
      const client = await window.MidenClient.createMock();
      await window.importStore(client.storeIdentifier(), storeData);
      const restored = await client.accounts.get(walletId);
      return restored?.to_commitment().toHex();
    }, exported);

    expect(restoredCommitment).toEqual(exported.commitment);
  });
});

test.describe("export and import account", () => {
  test("should export and import a private account", async ({ page }) => {
    const exported = await page.evaluate(async () => {
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

      return {
        walletId,
        faucetId,
        serialized,
        initialCommitment,
        initialBalance: initialBalance?.toString(),
        storeName: client.storeIdentifier(),
      };
    });

    await clearStore(page, exported.storeName);

    const restored = await page.evaluate(
      async ({ walletId, faucetId, serialized }) => {
        const client = await window.MidenClient.createMock();
        const bytes = new Uint8Array(serialized);
        const deserialized = window.AccountFile.deserialize(bytes);
        await client.accounts.import({ file: deserialized });

        const account = await client.accounts.get(walletId);
        return {
          restoredCommitment: account?.to_commitment().toHex(),
          restoredBalance: account
            ?.vault()
            .getBalance(window.AccountId.fromHex(faucetId))
            ?.toString(),
        };
      },
      exported
    );

    expect(restored.restoredCommitment).toEqual(exported.initialCommitment);
    expect(restored.restoredBalance).toEqual(exported.initialBalance);
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
    const exported = await page.evaluate(async () => {
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
      const serialized = Array.from(noteFile.serialize());

      return {
        originalNoteId: noteId,
        serialized,
        storeName: client.storeIdentifier(),
      };
    });

    await clearStore(page, exported.storeName);

    const importedNoteId = await page.evaluate(async ({ serialized }) => {
      const client = await window.MidenClient.createMock();
      const bytes = new Uint8Array(serialized);
      const deserialized = window.NoteFile.deserialize(bytes);
      const noteId = await client.notes.import(deserialized);
      return noteId.toString();
    }, exported);

    expect(importedNoteId).toBe(exported.originalNoteId);
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
      const mintTxId = await client.transactions.mint({
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
