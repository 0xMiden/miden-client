import { expect } from "chai";
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
} from "./webClientTestUtils";
import { Page } from "@playwright/test";
import {
  ConsumableNoteRecord,
  NoteConsumability,
} from "../dist/crates/miden_client_web";

const getConsumableNotes = async (testingPage: Page, accountId?: string) => {
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
    expect(noteId).to.be.undefined;
  });

  test("retrieve an input note that does exist", async ({ page }) => {
    const { consumedNoteId } = await setupConsumedNote(page);

    const { noteId } = await getInputNote(consumedNoteId, page);
    expect(noteId).to.equal(consumedNoteId);
  });
});

test.describe("get_input_notes", () => {
  test("note exists, note filter all", async ({ page }) => {
    const { consumedNoteId } = await setupConsumedNote(page);
    const { noteIds } = await getInputNotes(page);
    expect(noteIds).to.have.lengthOf.at.least(1);
    expect(noteIds).to.include(consumedNoteId);
  });
});

test.describe("get_consumable_notes", () => {
  test("filter by account", async ({ page }) => {
    const { createdNoteId: noteId1, accountId: accountId1 } =
      await setupMintedNote(page);

    const result = await getConsumableNotes(page, accountId1);
    expect(result).to.have.lengthOf(1);
    result.forEach((record: ConsumableNoteRecord) => {
      expect(record.noteConsumability()).to.have.lengthOf(1);
      expect(record.noteConsumability()[0].accountId).to.equal(accountId1);
      expect(record.noteConsumability()).to.equal(noteId1);
      expect(record.noteConsumability()[0].consumableAfterBlock).to.be
        .undefined;
    });
  });

  test("no filter by account", async ({ page }) => {
    const { createdNoteId: noteId1, accountId: accountId1 } =
      await setupMintedNote(page);
    const { createdNoteId: noteId2, accountId: accountId2 } =
      await setupMintedNote(page);

    const result = await getConsumableNotes(page);
    expect(
      result.map((r: ConsumableNoteRecord) => r.inputNoteRecord().id)
    ).to.include.members([noteId1, noteId2]);
    expect(
      result.map(
        (r: ConsumableNoteRecord) => r.noteConsumability()[0].accountId
      )
    ).to.include.members([accountId1, accountId2]);
    expect(result).to.have.lengthOf(2);
    const consumableRecord1 = result.find(
      (r: ConsumableNoteRecord) =>
        r.inputNoteRecord().id().toString() === noteId1
    );
    const consumableRecord2 = result.find(
      (r: ConsumableNoteRecord) =>
        r.inputNoteRecord().id().toString() === noteId2
    );

    consumableRecord1!!.consumability.forEach((c: ConsumableNoteRecord) => {
      expect(c.inputNoteRecord().id().toString()).to.equal(accountId1);
    });

    consumableRecord2!!.consumability.forEach((c: ConsumableNoteRecord) => {
      expect(c.inputNoteRecord().id().toString()).to.equal(accountId2);
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
    expect(consumableSender).to.have.lengthOf(1);
    expect(consumableSender[0].consumability[0].consumableAfterBlock).to.equal(
      recallHeight
    );
    expect(consumableRecipient).to.have.lengthOf(1);
    expect(consumableRecipient[0].consumability[0].consumableAfterBlock).to.be
      .undefined;
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
          .withOwnOutputNotes(new window.OutputNotesArray([outputNote]))
          .build();

        let transactionResult = await client.newTransaction(
          senderAccountId,
          transactionRequest
        );

        await client.submitTransaction(transactionResult);

        await window.helpers.waitForTransaction(
          transactionResult.executedTransaction().id().toHex()
        );

        let createdNoteId = transactionResult
          .createdNotes()
          .notes()[0]
          .id()
          .toString();

        let consumeTransactionRequest = client.newConsumeTransactionRequest([
          createdNoteId,
        ]);

        let consumeTransactionResult = await client.newTransaction(
          targetAccountId,
          consumeTransactionRequest
        );

        await client.submitTransaction(consumeTransactionResult);

        await window.helpers.waitForTransaction(
          consumeTransactionResult.executedTransaction().id().toHex()
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
        _targetId: senderId,
        _faucetId: faucetId,
      }
    );

    expect(result.senderAccountBalance).to.equal("990");
    expect(result.targetAccountBalance).to.equal("10");
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
          .withOwnOutputNotes(new window.OutputNotesArray([outputNote]))
          .build();

        let transactionResult = await client.newTransaction(
          senderAccountId,
          transactionRequest
        );

        await client.submitTransaction(transactionResult);

        await window.helpers.waitForTransaction(
          transactionResult.executedTransaction().id().toHex()
        );

        let createdNoteId = transactionResult
          .createdNotes()
          .notes()[0]
          .id()
          .toString();

        let consumeTransactionRequest = client.newConsumeTransactionRequest([
          createdNoteId,
        ]);

        let consumeTransactionResult = await client.newTransaction(
          targetAccountId,
          consumeTransactionRequest
        );

        await client.submitTransaction(consumeTransactionResult);

        await window.helpers.waitForTransaction(
          consumeTransactionResult.executedTransaction().id().toHex()
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

    expect(result.senderAccountBalance).to.equal("990");
    expect(result.targetAccountBalance).to.equal("10");
  });
});

// TODO:
test.describe("get_output_note", () => {});

test.describe("get_output_notes", () => {});
