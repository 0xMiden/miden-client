import test from "./playwright.global.setup";
import {
  badHexId,
  consumeTransaction,
  getSyncHeight,
  mintTransaction,
  sendTransaction,
  setupWalletAndFaucet,
  getInputNote,
  setupConsumedNote,
  getInputNotes,
  setupMintedNote,
  setupPublicConsumedNote,
} from "./webClientTestUtils";
import { Page, expect } from "@playwright/test";
import {
  ConsumableNoteRecord,
  NoteConsumability,
} from "../dist/crates/miden_client_web";

const getConsumableNotes = async (
  testingPage: Page,
  accountId?: string
): Promise<
  {
    noteId: string;
    consumability: {
      accountId: string;
      consumableAfterBlock: number | undefined;
    }[];
  }[]
> => {
  return await testingPage.evaluate(async (_accountId?: string) => {
    const client = window.client;
    let records;
    if (_accountId) {
      console.log({ _accountId });
      const accountId = window.AccountId.fromHex(_accountId);
      records = await client.getConsumableNotes(accountId);
    } else {
      records = await client.getConsumableNotes();
    }

    return records.map((record: ConsumableNoteRecord) => ({
      noteId: record.inputNoteRecord().id().toString(),
      consumability: record.noteConsumability().map((c) => ({
        accountId: c.accountId().toString(),
        consumableAfterBlock: c.consumableAfterBlock(),
      })),
    }));
  }, accountId);
};

test.describe("get_input_note", () => {
  test("retrieve input note that does not exist", async ({ page }) => {
    await setupWalletAndFaucet(page);
    const { noteId } = await getInputNote(badHexId, page);
    expect(noteId).toBeUndefined();
  });

  test("retrieve an input note that does exist", async ({ page }) => {
    const { consumedNoteId } = await setupConsumedNote(page);

    // Test both the existing client method and new RpcClient
    const { noteId } = await getInputNote(consumedNoteId, page);
    expect(noteId).toEqual(consumedNoteId);

    // Test RpcClient.getNotesById
    const rpcResult = await page.evaluate(async (_consumedNoteId: string) => {
      const endpoint = new window.Endpoint(window.rpcUrl);
      const rpcClient = new window.RpcClient(endpoint);

      const noteId = window.NoteId.fromHex(_consumedNoteId);
      const fetchedNotes = await rpcClient.getNotesById([noteId]);

      return fetchedNotes.map((note) => ({
        noteId: note.noteId.toString(),
        hasMetadata: !!note.metadata,
        noteType: note.noteType,
        hasInputNote: !!note.inputNote,
      }));
    }, consumedNoteId);

    // Assert on FetchedNote properties
    expect(rpcResult).toHaveLength(1);
    expect(rpcResult[0].noteId).toEqual(consumedNoteId);
    expect(rpcResult[0].hasMetadata).toBe(true);
    expect(rpcResult[0].hasInputNote).toBe(false); // Private notes don't include input_note
  });

  test("get note script by root", async ({ page }) => {
    await setupWalletAndFaucet(page);

    // First, we need to get a note script root from an existing note
    const { consumedNoteId } = await setupConsumedNote(page, true);

    // Get the note to extract its script root
    const noteData = await page.evaluate(async (_consumedNoteId: string) => {
      const endpoint = new window.Endpoint(window.rpcUrl);
      const rpcClient = new window.RpcClient(endpoint);

      const noteId = window.NoteId.fromHex(_consumedNoteId);
      const fetchedNotes = await rpcClient.getNotesById([noteId]);

      if (fetchedNotes.length > 0 && fetchedNotes[0].inputNote) {
        const scriptRoot = fetchedNotes[0].inputNote.note().script().root();
        return {
          scriptRoot: scriptRoot.toHex(),
          hasScript: true,
        };
      }

      return { scriptRoot: "", hasScript: false };
    }, consumedNoteId);

    // Test GetNoteScriptByRoot endpoint
    const retrievedScript = await page.evaluate(
      async (scriptRootHex: string) => {
        const endpoint = new window.Endpoint(window.rpcUrl);
        const rpcClient = new window.RpcClient(endpoint);

        const scriptRoot = window.Word.fromHex(scriptRootHex);
        const noteScript = await rpcClient.getNoteScriptByRoot(scriptRoot);

        return {
          hasScript: !!noteScript,
          scriptRoot: noteScript ? noteScript.root().toHex() : null,
        };
      },
      noteData.scriptRoot
    );

    expect(retrievedScript.hasScript).toBe(true);
    expect(retrievedScript.scriptRoot).toEqual(noteData.scriptRoot);
  });
});

test.describe("get_input_notes", () => {
  test("note exists, note filter all", async ({ page }) => {
    const { consumedNoteId } = await setupConsumedNote(page);
    const { noteIds } = await getInputNotes(page);
    expect(noteIds.length).toBeGreaterThanOrEqual(1);
    expect(noteIds).toContain(consumedNoteId);
  });
});

test.describe("get_consumable_notes", () => {
  test("filter by account", async ({ page }) => {
    const { createdNoteId: noteId1, accountId: accountId1 } =
      await setupMintedNote(page);

    const result = await getConsumableNotes(page, accountId1);
    expect(result).toHaveLength(1);
    result.forEach((record) => {
      expect(record.consumability).toHaveLength(1);
      expect(record.consumability[0].accountId).toBe(accountId1);
      expect(record.noteId).toBe(noteId1);
      expect(record.consumability[0].consumableAfterBlock).toBeUndefined();
    });
  });

  test("no filter by account", async ({ page }) => {
    const { createdNoteId: noteId1, accountId: accountId1 } =
      await setupMintedNote(page);
    const { createdNoteId: noteId2, accountId: accountId2 } =
      await setupMintedNote(page);

    const noteIds = new Set([noteId1, noteId2]);
    const accountIds = new Set([accountId1, accountId2]);
    const result = await getConsumableNotes(page);
    expect(noteIds).toEqual(new Set(result.map((r) => r.noteId)));
    expect(accountIds).toEqual(
      new Set(result.map((r) => r.consumability[0].accountId))
    );
    expect(result.length).toBeGreaterThanOrEqual(2);
    const consumableRecord1 = result.find((r) => r.noteId === noteId1);
    const consumableRecord2 = result.find((r) => r.noteId === noteId2);

    consumableRecord1!!.consumability.forEach((c) => {
      expect(c.accountId).toEqual(accountId1);
    });

    consumableRecord2!!.consumability.forEach((c) => {
      expect(c.accountId).toEqual(accountId2);
    });
  });

  test("p2ide consume after block", async ({ page }) => {
    const { accountId: senderAccountId, faucetId } =
      await setupWalletAndFaucet(page);
    const { accountId: targetAccountId } = await setupWalletAndFaucet(page);
    const recallHeight = (await getSyncHeight(page)) + 30;
    await sendTransaction(
      page,
      senderAccountId,
      targetAccountId,
      faucetId,
      recallHeight
    );

    const consumableRecipient = await getConsumableNotes(page, targetAccountId);
    const consumableSender = await getConsumableNotes(page, senderAccountId);
    expect(consumableSender.length).toBe(1);
    expect(consumableSender[0].consumability[0].consumableAfterBlock).toBe(
      recallHeight
    );
    expect(consumableRecipient.length).toBe(1);
    expect(
      consumableRecipient[0].consumability[0].consumableAfterBlock
    ).toBeUndefined();
  });
});

test.describe("createP2IDNote and createP2IDENote", () => {
  test("should create a proper consumable p2id note from the createP2IDNote function", async ({
    page,
  }) => {
    const { accountId: senderId, faucetId } = await setupWalletAndFaucet(page);
    const { accountId: targetId } = await setupWalletAndFaucet(page);

    const { createdNoteId } = await mintTransaction(
      page,
      senderId,
      faucetId,
      false,
      true
    );

    await consumeTransaction(page, senderId, faucetId, createdNoteId, false);

    const result = await page.evaluate(
      async ({ _senderId, _targetId, _faucetId }) => {
        let client = window.client;

        let senderAccountId = window.AccountId.fromHex(_senderId);
        let targetAccountId = window.AccountId.fromHex(_targetId);
        let faucetAccountId = window.AccountId.fromHex(_faucetId);

        let fungibleAsset = new window.FungibleAsset(
          faucetAccountId,
          BigInt(10)
        );
        let noteAssets = new window.NoteAssets([fungibleAsset]);
        let p2IdNote = window.Note.createP2IDNote(
          senderAccountId,
          targetAccountId,
          noteAssets,
          window.NoteType.Public,
          new window.Felt(0n)
        );

        let outputNote = window.OutputNote.full(p2IdNote);

        let transactionRequest = new window.TransactionRequestBuilder()
          .withOwnOutputNotes(
            new window.MidenArrays.OutputNoteArray([outputNote])
          )
          .build();

        let transactionUpdate = await window.helpers.executeAndApplyTransaction(
          senderAccountId,
          transactionRequest
        );

        await window.helpers.waitForTransaction(
          transactionUpdate.executedTransaction().id().toHex()
        );

        let createdNoteId = transactionUpdate
          .executedTransaction()
          .outputNotes()
          .notes()[0]
          .id()
          .toString();

        let consumeTransactionRequest = client.newConsumeTransactionRequest([
          createdNoteId,
        ]);

        let consumeTransactionUpdate =
          await window.helpers.executeAndApplyTransaction(
            targetAccountId,
            consumeTransactionRequest
          );

        await window.helpers.waitForTransaction(
          consumeTransactionUpdate.executedTransaction().id().toHex()
        );

        let senderAccountBalance = (await client.getAccount(senderAccountId))
          ?.vault()
          .getBalance(faucetAccountId)
          .toString();
        let targetAccountBalance = (await client.getAccount(targetAccountId))
          ?.vault()
          .getBalance(faucetAccountId)
          .toString();

        return {
          senderAccountBalance: senderAccountBalance,
          targetAccountBalance: targetAccountBalance,
        };
      },
      {
        _senderId: senderId,
        _targetId: targetId,
        _faucetId: faucetId,
      }
    );

    expect(result.senderAccountBalance).toEqual("990");
    expect(result.targetAccountBalance).toEqual("10");
  });

  test("should create a proper consumable p2ide note from the createP2IDENote function", async ({
    page,
  }) => {
    const { accountId: senderId, faucetId } = await setupWalletAndFaucet(page);
    const { accountId: targetId } = await setupWalletAndFaucet(page);

    const { createdNoteId } = await mintTransaction(
      page,
      senderId,
      faucetId,
      false,
      true
    );

    await consumeTransaction(page, senderId, faucetId, createdNoteId, false);

    const result = await page.evaluate(
      async ({ _senderId, _targetId, _faucetId }) => {
        let client = window.client;

        console.log(_senderId, _targetId, _faucetId);
        let senderAccountId = window.AccountId.fromHex(_senderId);
        let targetAccountId = window.AccountId.fromHex(_targetId);
        let faucetAccountId = window.AccountId.fromHex(_faucetId);

        let fungibleAsset = new window.FungibleAsset(
          faucetAccountId,
          BigInt(10)
        );
        let noteAssets = new window.NoteAssets([fungibleAsset]);
        let p2IdeNote = window.Note.createP2IDENote(
          senderAccountId,
          targetAccountId,
          noteAssets,
          null,
          null,
          window.NoteType.Public,
          new window.Felt(0n)
        );

        let outputNote = window.OutputNote.full(p2IdeNote);

        let transactionRequest = new window.TransactionRequestBuilder()
          .withOwnOutputNotes(
            new window.MidenArrays.OutputNoteArray([outputNote])
          )
          .build();

        let transactionUpdate = await window.helpers.executeAndApplyTransaction(
          senderAccountId,
          transactionRequest
        );

        await window.helpers.waitForTransaction(
          transactionUpdate.executedTransaction().id().toHex()
        );

        let createdNoteId = transactionUpdate
          .executedTransaction()
          .outputNotes()
          .notes()[0]
          .id()
          .toString();

        let consumeTransactionRequest = client.newConsumeTransactionRequest([
          createdNoteId,
        ]);

        let consumeTransactionUpdate =
          await window.helpers.executeAndApplyTransaction(
            targetAccountId,
            consumeTransactionRequest
          );

        await window.helpers.waitForTransaction(
          consumeTransactionUpdate.executedTransaction().id().toHex()
        );

        let senderAccountBalance = (await client.getAccount(senderAccountId))
          ?.vault()
          .getBalance(faucetAccountId)
          .toString();
        let targetAccountBalance = (await client.getAccount(targetAccountId))
          ?.vault()
          .getBalance(faucetAccountId)
          .toString();

        return {
          senderAccountBalance: senderAccountBalance,
          targetAccountBalance: targetAccountBalance,
        };
      },
      {
        _senderId: senderId,
        _targetId: targetId,
        _faucetId: faucetId,
      }
    );

    expect(result.senderAccountBalance).toEqual("990");
    expect(result.targetAccountBalance).toEqual("10");
  });
});

// TODO:
test.describe("get_output_note", () => {});

test.describe("get_output_notes", () => {});
