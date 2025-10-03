// @ts-nocheck
import test from "./playwright.global.setup";
import { Page, expect } from "@playwright/test";

const mockChainTest = async (testingPage: Page) => {
  return await testingPage.evaluate(async () => {
    const client = await window.MockWebClient.createClient();
    await client.syncState();

    const executeAndApply = async (accountId, transactionRequest) => {
      const pipeline = await client.executeTransactionPipeline(
        accountId,
        transactionRequest
      );

      await pipeline.proveTransaction(
        window.TransactionProver.newLocalProver()
      );
      const submittedUpdate = await pipeline.submitProvenTransaction();
      await client.applyTransaction(submittedUpdate);

      return pipeline.getTransactionUpdate();
    };

    const account = await client.newWallet(
      window.AccountStorageMode.private(),
      true
    );
    const faucetAccount = await client.newFaucet(
      window.AccountStorageMode.private(),
      false,
      "DAG",
      8,
      BigInt(10000000)
    );

    const mintTransactionRequest = await client.newMintTransactionRequest(
      account.id(),
      faucetAccount.id(),
      window.NoteType.Public,
      BigInt(1000)
    );

    const mintTransactionUpdate = await executeAndApply(
      faucetAccount.id(),
      mintTransactionRequest
    );
    await client.proveBlock();
    await client.syncState();

    const consumeTransactionRequest = client.newConsumeTransactionRequest([
      mintTransactionUpdate
        .executedTransaction()
        .outputNotes()
        .notes()[0]
        .id()
        .toString(),
    ]);
    const consumeTransactionUpdate = await executeAndApply(
      account.id(),
      consumeTransactionRequest
    );
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
