import { expect } from "chai";
import { testingPage } from "./mocha.global.setup.mjs";
import {
  badHexId,
  consumeTransaction,
  getSyncHeight,
  mintTransaction,
  sendTransaction,
  setupWalletAndFaucet,
} from "./webClientTestUtils";

const getInputNote = async (noteId: string) => {
  return await testingPage.evaluate(async (_noteId) => {
    const client = window.client;
    const note = await client.getInputNote(_noteId);
    return {
      noteId: note ? note.id().toString() : undefined,
    };
  }, noteId);
};

// TODO: Figure out a way to easily pass NoteFilters into the tests
const getInputNotes = async () => {
  return await testingPage.evaluate(async () => {
    const client = window.client;
    const filter = new window.NoteFilter(window.NoteFilterTypes.All);
    const notes = await client.getInputNotes(filter);
    return {
      noteIds: notes.map((note) => note.id().toString()),
    };
  });
};

const setupMintedNote = async () => {
  const { accountId, faucetId } = await setupWalletAndFaucet();
  const { createdNoteId } = await mintTransaction(accountId, faucetId);

  return { createdNoteId, accountId, faucetId };
};

export const setupConsumedNote = async () => {
  const { createdNoteId, accountId, faucetId } = await setupMintedNote();
  await consumeTransaction(accountId, faucetId, createdNoteId);

  return {
    consumedNoteId: createdNoteId,
    accountId: accountId,
    faucetId: faucetId,
  };
};

const getConsumableNotes = async (accountId?: string) => {
  return await testingPage.evaluate(async (_accountId) => {
    const client = window.client;
    let records;
    if (_accountId) {
      console.log({ _accountId });
      const accountId = window.AccountId.fromHex(_accountId);
      records = await client.getConsumableNotes(accountId);
    } else {
      records = await client.getConsumableNotes();
    }

    return records.map((record) => ({
      noteId: record.inputNoteRecord().id().toString(),
      consumability: record.noteConsumability().map((c) => ({
        accountId: c.accountId().toString(),
        consumableAfterBlock: c.consumableAfterBlock(),
      })),
    }));
  }, accountId);
};

describe("get_input_note", () => {
  it("retrieve input note that does not exist", async () => {
    await setupWalletAndFaucet();
    const { noteId } = await getInputNote(badHexId);
    await expect(noteId).to.be.undefined;
  });

  it("retrieve an input note that does exist", async () => {
    const { consumedNoteId } = await setupConsumedNote();

    // Test both the existing client method and new RpcClient
    const { noteId } = await getInputNote(consumedNoteId);
    expect(noteId).to.equal(consumedNoteId);

    // Test RpcClient.getNotesById
    const rpcResult = await testingPage.evaluate(
      async (_consumedNoteId: string) => {
        // NOTE: this assumes the node is running on localhost
        const endpoint = new window.Endpoint("http://localhost:57291");
        const rpcClient = new window.RpcClient(endpoint);

        const noteId = window.NoteId.fromHex(_consumedNoteId);
        const fetchedNotes = await rpcClient.getNotesById([noteId]);

        return fetchedNotes.map((note) => ({
          noteId: note.noteId.toString(),
          hasMetadata: !!note.metadata,
          noteType: note.noteType,
          hasInputNote: !!note.inputNote,
        }));
      },
      consumedNoteId
    );

    // Assert on FetchedNote properties
    expect(rpcResult).to.have.lengthOf(1);
    expect(rpcResult[0].noteId).to.equal(consumedNoteId);
    expect(rpcResult[0].hasMetadata).to.be.true;
    expect(rpcResult[0].hasInputNote).to.be.false; // Private notes don't include input_note
  });
});

describe("get_input_notes", () => {
  it("note exists, note filter all", async () => {
    const { consumedNoteId } = await setupConsumedNote();
    const { noteIds } = await getInputNotes();
    expect(noteIds).to.have.lengthOf.at.least(1);
    expect(noteIds).to.include(consumedNoteId);
  });
});

describe("get_consumable_notes", () => {
  it("filter by account", async () => {
    const { createdNoteId: noteId1, accountId: accountId1 } =
      await setupMintedNote();

    const result = await getConsumableNotes(accountId1);
    expect(result).to.have.lengthOf(1);
    result.forEach((record) => {
      expect(record.consumability).to.have.lengthOf(1);
      expect(record.consumability[0].accountId).to.equal(accountId1);
      expect(record.noteId).to.equal(noteId1);
      expect(record.consumability[0].consumableAfterBlock).to.be.undefined;
    });
  });

  it("no filter by account", async () => {
    const { createdNoteId: noteId1, accountId: accountId1 } =
      await setupMintedNote();
    const { createdNoteId: noteId2, accountId: accountId2 } =
      await setupMintedNote();

    const result = await getConsumableNotes();
    expect(result.map((r) => r.noteId)).to.include.members([noteId1, noteId2]);
    expect(result.map((r) => r.consumability[0].accountId)).to.include.members([
      accountId1,
      accountId2,
    ]);
    expect(result).to.have.lengthOf(2);
    const consumableRecord1 = result.find((r) => r.noteId === noteId1);
    const consumableRecord2 = result.find((r) => r.noteId === noteId2);

    consumableRecord1!!.consumability.forEach((c) => {
      expect(c.accountId).to.equal(accountId1);
    });

    consumableRecord2!!.consumability.forEach((c) => {
      expect(c.accountId).to.equal(accountId2);
    });
  });

  it("p2ide consume after block", async () => {
    const { accountId: senderAccountId, faucetId } =
      await setupWalletAndFaucet();
    const { accountId: targetAccountId } = await setupWalletAndFaucet();
    const recallHeight = (await getSyncHeight()) + 30;
    await sendTransaction(
      senderAccountId,
      targetAccountId,
      faucetId,
      recallHeight
    );

    const consumableRecipient = await getConsumableNotes(targetAccountId);
    const consumableSender = await getConsumableNotes(senderAccountId);
    expect(consumableSender).to.have.lengthOf(1);
    expect(consumableSender[0].consumability[0].consumableAfterBlock).to.equal(
      recallHeight
    );
    expect(consumableRecipient).to.have.lengthOf(1);
    expect(consumableRecipient[0].consumability[0].consumableAfterBlock).to.be
      .undefined;
  });
});

describe("createP2IDNote and createP2IDENote", () => {
  it("should create a proper consumable p2id note from the createP2IDNote function", async () => {
    const { accountId: senderId, faucetId } = await setupWalletAndFaucet();
    const { accountId: targetId } = await setupWalletAndFaucet();

    const { createdNoteId } = await mintTransaction(
      senderId,
      faucetId,
      false,
      true
    );

    await consumeTransaction(senderId, faucetId, createdNoteId, false);

    const result = await testingPage.evaluate(
      async (_senderId: string, _targetId: string, _faucetId: string) => {
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
      senderId,
      targetId,
      faucetId
    );

    expect(result.senderAccountBalance).to.equal("990");
    expect(result.targetAccountBalance).to.equal("10");
  });

  it("should create a proper consumable p2ide note from the createP2IDENote function", async () => {
    const { accountId: senderId, faucetId } = await setupWalletAndFaucet();
    const { accountId: targetId } = await setupWalletAndFaucet();

    const { createdNoteId } = await mintTransaction(
      senderId,
      faucetId,
      false,
      true
    );

    await consumeTransaction(senderId, faucetId, createdNoteId, false);

    const result = await testingPage.evaluate(
      async (_senderId: string, _targetId: string, _faucetId: string) => {
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
      senderId,
      targetId,
      faucetId
    );

    expect(result.senderAccountBalance).to.equal("990");
    expect(result.targetAccountBalance).to.equal("10");
  });
});

// TODO:
describe("get_output_note", () => {});

describe("get_output_notes", () => {});
