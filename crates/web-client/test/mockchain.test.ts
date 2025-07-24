import { expect } from "chai";
import { testingPage } from "./mocha.global.setup.mjs";
import {
  clearStore,
  createNewFaucet,
  createNewWallet,
  fundAccountFromFaucet,
  getAccount,
  getAccountBalance,
  StorageMode,
} from "./webClientTestUtils";

const mockChainTest = async () => {
  return await testingPage.evaluate(async () => {
    const client = await window.WebClient.createMockedClient();
    await client.syncState();

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

    const mintTransactionResult = await client.newTransaction(
      faucetAccount.id(),
      mintTransactionRequest
    );

    await client.submitTransaction(mintTransactionResult);
    await client.proveBlock();
    await client.syncState();

    const consumeTransactionRequest = client.newConsumeTransactionRequest([
      mintTransactionResult.createdNotes().notes()[0].id().toString(),
    ]);
    const consumeTransactionResult = await client.newTransaction(
      account.id(),
      consumeTransactionRequest
    );

    await client.submitTransaction(consumeTransactionResult);
    await client.proveBlock();
    await client.syncState();

    const changedTargetAccount = await client.getAccount(account.id());

    return changedTargetAccount
      .vault()
      .getBalance(faucetAccount.id())
      .toString();
  });
};

describe("mock chain tests", () => {
  it("send transaction with mock chain completes successfully", async () => {
    let finalBalance = await mockChainTest();
    expect(finalBalance).to.equal("1000");
  });
});
