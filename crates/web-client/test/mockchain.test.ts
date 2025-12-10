// @ts-nocheck
import test from "./playwright.global.setup";
import { Page, expect } from "@playwright/test";

const mockChainTest = async (testingPage: Page) => {
  return await testingPage.evaluate(async () => {
    // Mockchain tests share the same database with the rest of the
    await new Promise<void>((resolve, reject) => {
      const request = indexedDB.deleteDatabase("MidenClientDB");
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

    const consumeTransactionRequest = client.newConsumeTransactionRequest([
      mintedNoteId,
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
