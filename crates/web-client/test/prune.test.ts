import test from "./playwright.global.setup";
import { expect } from "@playwright/test";
import {
  setupWalletAndFaucet,
  mintAndConsumeTransaction,
  sendTransaction,
  consumeTransaction,
} from "./webClientTestUtils";
import { Note } from "../dist/crates/miden_client_web";

// PRUNE ACCOUNT HISTORY TESTS
// =======================================================================================================

interface PruneTestResult {
  prunedCount: number;
  secondPrunedCount: number;
  firstWalletNonce: string;
  secondWalletNonce: string;
  faucetNonce: string;
}

const pruneAccountHistoryTest = async (
  testingPage: any
): Promise<PruneTestResult> => {
  return await testingPage.evaluate(async () => {
    const client = window.client;

    // Create first wallet and faucet
    const firstWallet = await client.newWallet(
      window.AccountStorageMode.private(),
      true,
      0
    );
    const faucetAccount = await client.newFaucet(
      window.AccountStorageMode.private(),
      false,
      "DAG",
      8,
      BigInt(10000000)
    );

    // Create second wallet
    const secondWallet = await client.newWallet(
      window.AccountStorageMode.private(),
      true,
      0
    );

    await client.syncState();

    // Mint tokens to first wallet (faucet nonce: 0 -> 1)
    const mintTransactionRequest = client.newMintTransactionRequest(
      firstWallet.id(),
      faucetAccount.id(),
      window.NoteType.Private,
      BigInt(1000)
    );

    const mintTransactionUpdate =
      await window.helpers.executeAndApplyTransaction(
        faucetAccount.id(),
        mintTransactionRequest
      );

    const mintedNote = mintTransactionUpdate
      .executedTransaction()
      .outputNotes()
      .notes()[0]
      .intoFull();

    await window.helpers.waitForTransaction(
      mintTransactionUpdate.executedTransaction().id().toHex()
    );

    // Consume minted note (first wallet nonce: 0 -> 1)
    let noteAndArgs = new window.NoteAndArgs(mintedNote, null);
    let noteAndArgsArray = new window.NoteAndArgsArray([noteAndArgs]);

    let consumeRequest = new window.TransactionRequestBuilder()
      .withInputNotes(noteAndArgsArray)
      .build();

    let consumeTransactionUpdate =
      await window.helpers.executeAndApplyTransaction(
        firstWallet.id(),
        consumeRequest
      );

    await window.helpers.waitForTransaction(
      consumeTransactionUpdate.executedTransaction().id().toHex()
    );

    // Send tokens from first wallet to second wallet (first wallet nonce: 1 -> 2)
    let sendTransactionRequest = client.newSendTransactionRequest(
      firstWallet.id(),
      secondWallet.id(),
      faucetAccount.id(),
      window.NoteType.Public,
      BigInt(100),
      null,
      null
    );

    let sendTransactionUpdate = await window.helpers.executeAndApplyTransaction(
      firstWallet.id(),
      sendTransactionRequest
    );

    const sentNoteId = sendTransactionUpdate
      .executedTransaction()
      .outputNotes()
      .notes()[0]
      .id()
      .toString();

    await window.helpers.waitForTransaction(
      sendTransactionUpdate.executedTransaction().id().toHex()
    );

    // Consume sent note on second wallet (second wallet nonce: 0 -> 1)
    const inputNoteRecord = await client.getInputNote(sentNoteId);
    if (!inputNoteRecord) {
      throw new Error(`Note with ID ${sentNoteId} not found`);
    }

    const sentNote = inputNoteRecord.toNote();
    noteAndArgs = new window.NoteAndArgs(sentNote, null);
    noteAndArgsArray = new window.NoteAndArgsArray([noteAndArgs]);

    consumeRequest = new window.TransactionRequestBuilder()
      .withInputNotes(noteAndArgsArray)
      .build();

    consumeTransactionUpdate = await window.helpers.executeAndApplyTransaction(
      secondWallet.id(),
      consumeRequest
    );

    await window.helpers.waitForTransaction(
      consumeTransactionUpdate.executedTransaction().id().toHex()
    );

    // Now we have:
    // - Faucet: 2 states (nonce 0, nonce 1) -> 1 old state to prune
    // - First wallet: 3 states (nonce 0, 1, 2) -> 2 old states to prune
    // - Second wallet: 2 states (nonce 0, 1) -> 1 old state to prune
    // Total: 4 old states to prune

    // Prune old account states
    const prunedCount = await client.pruneAccountHistory();

    // Verify accounts still work with correct nonces
    const firstWalletAccount = await client.getAccount(firstWallet.id());
    const secondWalletAccount = await client.getAccount(secondWallet.id());
    const faucetAccountAfter = await client.getAccount(faucetAccount.id());

    // Second prune should return 0
    const secondPrunedCount = await client.pruneAccountHistory();

    return {
      prunedCount,
      secondPrunedCount,
      firstWalletNonce: firstWalletAccount!.nonce().toString(),
      secondWalletNonce: secondWalletAccount!.nonce().toString(),
      faucetNonce: faucetAccountAfter!.nonce().toString(),
    };
  });
};

test.describe("prune account history tests", () => {
  test("prune account history removes old states correctly", async ({
    page,
  }) => {
    const result = await pruneAccountHistoryTest(page);

    // Should have pruned 4 old states
    expect(result.prunedCount).toEqual(4);

    // Second prune should return 0
    expect(result.secondPrunedCount).toEqual(0);

    // Verify nonces are correct after pruning
    expect(result.firstWalletNonce).toEqual("2");
    expect(result.secondWalletNonce).toEqual("1");
    expect(result.faucetNonce).toEqual("1");
  });
});
