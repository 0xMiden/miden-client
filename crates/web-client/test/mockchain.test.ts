// @ts-nocheck
import test from "./playwright.global.setup";
import { Page, expect } from "@playwright/test";

const mockChainTest = async (testingPage: Page) => {
  return await testingPage.evaluate(async () => {
    // Web Client tests share the same browser database "MidenClientDB"
    // Across all tests. This is error prone as the mockChainTest will
    // run this test with a previously loaded DB which doesn't correspond
    // to the actual context.
    // https://github.com/0xMiden/miden-client/issues/1611
    await new Promise<void>((resolve, reject) => {
      const request = indexedDB.deleteDatabase("MidenClientDB_mlcl");
      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
      request.onblocked = () => resolve();
    });

    const client = await window.MockWebClient.createClient();
    await client.syncState();

    const account = await client.newWallet(
      window.AccountStorageMode.private(),
      true,
      0
    );
    const faucetAccount = await client.newFaucet(
      window.AccountStorageMode.private(),
      false,
      "DAG",
      8,
      BigInt(10000000),
      0
    );

    const mintTransactionRequest = await client.newMintTransactionRequest(
      account.id(),
      faucetAccount.id(),
      window.NoteType.Public,
      BigInt(1000)
    );

    const mintTransactionId = await client.submitNewTransaction(
      faucetAccount.id(),
      mintTransactionRequest
    );
    await client.proveBlock();
    await client.syncState();

    const [mintTransactionRecord] = await client.getTransactions(
      window.TransactionFilter.ids([mintTransactionId])
    );
    if (!mintTransactionRecord) {
      throw new Error("Mint transaction record not found");
    }

    const mintedNoteId = mintTransactionRecord
      .outputNotes()
      .notes()[0]
      .id()
      .toString();

    const mintedNoteRecord = await client.getInputNote(mintedNoteId);
    if (!mintedNoteRecord) {
      throw new Error(`Note with ID ${mintedNoteId} not found`);
    }

    const mintedNote = mintedNoteRecord.toNote();
    const consumeTransactionRequest = client.newConsumeTransactionRequest([
      mintedNote,
    ]);
    await client.submitNewTransaction(account.id(), consumeTransactionRequest);
    await client.proveBlock();
    await client.syncState();

    const changedTargetAccount = await client.getAccount(account.id());

    return changedTargetAccount
      .vault()
      .getBalance(faucetAccount.id())
      .toString();
  });
};

test.describe("mock chain tests", () => {
  test("send transaction with mock chain completes successfully", async ({
    page,
  }) => {
    let finalBalance = await mockChainTest(page);
    expect(finalBalance).toEqual("1000");
  });
});
