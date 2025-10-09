// TODO: Rename this / figure out rebasing with the other feature which has import tests

import test from "./playwright.global.setup";
import { Page, expect } from "@playwright/test";
import {
  clearStore,
  createNewFaucet,
  createNewWallet,
  fundAccountFromFaucet,
  getAccountBalance,
  setupMintedNote,
  setupWalletAndFaucet,
  StorageMode,
} from "./webClientTestUtils";

const exportDb = async (page: Page) => {
  return await page.evaluate(async () => {
    const client = window.client;
    const db = await client.exportStore();
    const serialized = JSON.stringify(db);
    return serialized;
  });
};

const importDb = async (db: any, page: Page) => {
  return await page.evaluate(async (_db) => {
    const client = window.client;
    await client.forceImportStore(_db);
  }, db);
};

const getAccount = async (accountId: string, page: Page) => {
  return await page.evaluate(async (_accountId) => {
    const client = window.client;
    const accountId = window.AccountId.fromHex(_accountId);
    const account = await client.getAccount(accountId);
    return {
      accountId: account?.id().toString(),
      accountCommitment: account?.commitment().toHex(),
    };
  }, accountId);
};

const exportAccount = async (testingPage: Page, accountId: string) => {
  return await testingPage.evaluate(async (_accountId) => {
    const client = window.client;
    const accountId = window.AccountId.fromHex(_accountId);
    const accountFile = await client.exportAccountFile(accountId);
    return Array.from(accountFile.serialize());
  }, accountId);
};

const importAccount = async (testingPage: Page, accountBytes: number[]) => {
  return await testingPage.evaluate(async (_accountBytes) => {
    const client = window.client;
    const bytes = new Uint8Array(_accountBytes);
    const accountFile = window.AccountFile.deserialize(bytes);
    await client.importAccountFile(accountFile);
  }, accountBytes);
};

test.describe("export and import the db", () => {
  test("export db with an account, find the account when re-importing", async ({
    page,
  }) => {
    const { accountCommitment: initialAccountCommitment, accountId } =
      await setupWalletAndFaucet(page);
    const dbDump = await exportDb(page);

    await clearStore(page);

    await importDb(dbDump, page);

    const { accountCommitment } = await getAccount(accountId, page);

    expect(accountCommitment).toEqual(initialAccountCommitment);
  });
});

test.describe("export and import account", () => {
  test("should export and import a private account", async ({ page }) => {
    const walletSeed = new Uint8Array(32);
    crypto.getRandomValues(walletSeed);

    const mutable = false;
    const storageMode = StorageMode.PRIVATE;

    const initialWallet = await createNewWallet(page, {
      storageMode,
      mutable,
      walletSeed,
    });
    const faucet = await createNewFaucet(page);

    const { targetAccountBalance: initialBalance } =
      await fundAccountFromFaucet(page, initialWallet.id, faucet.id);
    const { accountCommitment: initialCommitment } = await getAccount(
      initialWallet.id,
      page
    );
    const exportedAccount = await exportAccount(page, initialWallet.id);
    await clearStore(page);

    await importAccount(page, exportedAccount);

    const { accountCommitment: restoredCommitment } = await getAccount(
      initialWallet.id,
      page
    );

    const restoredBalance = await getAccountBalance(
      page,
      initialWallet.id,
      faucet.id
    );

    expect(restoredCommitment).toEqual(initialCommitment);
    expect(restoredBalance.toString()).toEqual(initialBalance);
  });
});

test.describe("export and import note", () => {
  const exportTypes = [
    ["Id", "NoteId"],
    ["Full", "NoteWithProof"],
    ["Details", "NoteDetails"],
  ];

  const exportNote = async (
    testingPage: Page,
    noteId: string,
    exportType: string
  ) => {
    return await testingPage.evaluate(
      async ({ noteId, exportType }) => {
        const noteFile = await window.client.exportNoteFile(noteId, exportType);
        return noteFile.noteType();
      },
      { noteId, exportType }
    );
  };

  const exportNoteSerialized = async (
    testingPage: Page,
    noteId: string,
    exportType: string
  ) => {
    return await testingPage.evaluate(
      async ({ noteId, exportType }) => {
        const noteFile = await window.client.exportNoteFile(noteId, exportType);
        return noteFile.serialize();
      },
      { noteId, exportType }
    );
  };

  const importSerializedNote = async (
    testingPage: Page,
    serializedNote: Uint8Array
  ) => {
    return await testingPage.evaluate(
      async ({ serializedNote }) => {
        const noteFile = window.NoteFile.deserialize(serializedNote);
        const importedNoteId = window.client.importNoteFile(noteFile);
        return importedNoteId;
      },
      { serializedNote }
    );
  };

  exportTypes.forEach(([exportType, expectedNoteType]) => {
    test(`export note as note file -- export type: ${exportType}`, async ({
      page,
    }) => {
      const { createdNoteId: noteId } = await setupMintedNote(page);

      await expect(exportNote(page, noteId, exportType)).resolves.toBe(
        expectedNoteType
      );
    });
  });

  test(`exporting non-existing note fails`, async ({ page }) => {
    // Random note id taken from testnet
    const noteId =
      "0x60b06dbb6c7435ab1d439df972e483bca43bc21654dce2611de98ec3896beaed";
    await expect(exportNote(page, noteId, "Full")).rejects.toThrowError(
      "No output note found"
    );
  });

  test(`exporting and then importing note`, async ({ page }) => {
    const { createdNoteId: noteId } = await setupMintedNote(page);

    const serializedNoteFile = await exportNoteSerialized(page, noteId, "Full");

    // Clear store and assert that the output note cannot be found
    await clearStore(page);
    await expect(async () => {
      return await page.evaluate(
        async ({ noteId }) => {
          return await window.client.getOutputNote(noteId);
        },
        { noteId }
      );
    }).rejects.toThrow("Note not found");

    await expect(importSerializedNote(page, serializedNoteFile)).resolves.toBe(
      noteId
    );
  });
});
