// @ts-nocheck
import test from "./playwright.global.setup";
import { Page, expect } from "@playwright/test";

const pruneAccountHistoryTest = async (testingPage: Page) => {
  return await testingPage.evaluate(async () => {
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

    // Execute a mint transaction to create account history
    const mintTransactionRequest = await client.newMintTransactionRequest(
      account.id(),
      faucetAccount.id(),
      window.NoteType.Public,
      BigInt(1000)
    );
    await client.submitNewTransaction(
      faucetAccount.id(),
      mintTransactionRequest
    );
    await client.proveBlock();
    await client.syncState();

    // Consume the minted note to create more history
    const [mintTransactionRecord] = await client.getTransactions(
      window.TransactionFilter.all()
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

    const consumeTransactionRequest = client.newConsumeTransactionRequest([
      mintedNoteRecord.toNote(),
    ]);
    await client.submitNewTransaction(account.id(), consumeTransactionRequest);
    await client.proveBlock();
    await client.syncState();

    // Get the account's commitment before pruning
    const accountBeforePrune = await client.getAccount(account.id());
    const commitmentBeforePrune = accountBeforePrune.commitment().toHex();

    // Prune old account states
    const prunedCount = await client.pruneAccountHistory();

    // Verify the latest state is still correct after pruning
    const accountAfterPrune = await client.getAccount(account.id());
    const commitmentAfterPrune = accountAfterPrune.commitment().toHex();

    return {
      prunedCount,
      commitmentBeforePrune,
      commitmentAfterPrune,
    };
  });
};

test.describe("prune account history tests", () => {
  test("prune old account states preserves latest state", async ({ page }) => {
    const result = await pruneAccountHistoryTest(page);
    expect(result.prunedCount).toBeGreaterThan(0);
    expect(result.commitmentBeforePrune).toEqual(result.commitmentAfterPrune);
  });
});
