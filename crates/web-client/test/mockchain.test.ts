// @ts-nocheck
import { test, expect } from "./test-setup";

test.describe("mock chain tests", () => {
  test.describe.configure({ timeout: 720000 });

  test("send transaction with mock chain completes successfully", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk }) => {
      await client.syncState();

      const account = await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const faucetAccount = await client.newFaucet(
        sdk.AccountStorageMode.private(),
        false,
        "DAG",
        8,
        sdk.u64(10000000),
        sdk.AuthScheme.AuthRpoFalcon512
      );

      const mintTransactionRequest = await client.newMintTransactionRequest(
        account.id(),
        faucetAccount.id(),
        sdk.NoteType.Public,
        sdk.u64(1000)
      );

      const mintTransactionId = await client.submitNewTransaction(
        faucetAccount.id(),
        mintTransactionRequest
      );
      await client.proveBlock();
      await client.syncState();

      const [mintTransactionRecord] = await client.getTransactions(
        sdk.TransactionFilter.ids([mintTransactionId])
      );

      const mintedNoteId = mintTransactionRecord
        .outputNotes()
        .notes()[0]
        .id()
        .toString();

      const mintedNoteRecord = await client.getInputNote(mintedNoteId);

      const mintedNote = mintedNoteRecord.toNote();
      const consumeTransactionRequest = client.newConsumeTransactionRequest([
        mintedNote,
      ]);
      await client.submitNewTransaction(
        account.id(),
        consumeTransactionRequest
      );
      await client.proveBlock();
      await client.syncState();

      const changedTargetAccount = await client.getAccount(account.id());

      const finalBalance = changedTargetAccount
        .vault()
        .getBalance(faucetAccount.id())
        .toString();

      return {
        mintTransactionRecordDefined: mintTransactionRecord !== undefined,
        mintedNoteRecordDefined: mintedNoteRecord !== undefined,
        finalBalance,
      };
    });

    expect(result.mintTransactionRecordDefined).toBe(true);
    expect(result.mintedNoteRecordDefined).toBe(true);
    expect(result.finalBalance).toEqual("1000");
  });
});
