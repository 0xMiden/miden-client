import { test, expect, sdk } from "./setup.ts";

test.describe("mock chain tests", () => {
  test("mint and consume transaction completes successfully", async ({
    mockClient,
  }) => {
    const { client } = mockClient;

    await client.syncStateImpl();

    // Create wallet and faucet
    const wallet = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512
    );
    const faucet = await client.newFaucet(
      sdk.AccountStorageMode.private(),
      false,
      "DAG",
      8,
      10000000,
      sdk.AuthScheme.AuthRpoFalcon512
    );

    // Mint tokens
    const mintRequest = await client.newMintTransactionRequest(
      wallet.id(),
      faucet.id(),
      sdk.NoteType.Public,
      1000
    );
    const mintTxId = await client.submitNewTransaction(
      faucet.id(),
      mintRequest
    );
    await client.proveBlock();
    await client.syncStateImpl();

    // Verify mint transaction exists
    const [mintRecord] = await client.getTransactions(
      sdk.TransactionFilter.ids([mintTxId])
    );
    expect(mintRecord).toBeDefined();

    // Get the minted note
    const mintedNoteId = mintRecord.outputNotes().notes()[0].id().toString();

    const noteRecord = await client.getInputNote(mintedNoteId);
    expect(noteRecord).toBeDefined();

    // Consume the note
    const note = noteRecord.toNote();
    const consumeRequest = client.newConsumeTransactionRequest([note]);
    await client.submitNewTransaction(wallet.id(), consumeRequest);
    await client.proveBlock();
    await client.syncStateImpl();

    // Check final balance
    const account = await client.getAccount(wallet.id());
    const balance = account.vault().getBalance(faucet.id()).toString();
    expect(balance).toBe("1000");
  });
});
