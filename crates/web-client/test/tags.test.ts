// @ts-nocheck
import { test, expect, executeAndApplyTransaction } from "./test-setup";

// ADD_TAG TESTS
// =======================================================================================================

test.describe("add_tag tests", () => {
  test("adds a tag to the system", async ({ client }) => {
    const tag = "123";
    await client.addTag(tag);
    const tags = await client.listTags();
    expect(tags).toContain(tag);
  });
});

// REMOVE_TAG TESTS
// =======================================================================================================

test.describe("remove_tag tests", () => {
  test("removes a tag from the system", async ({ client }) => {
    const tag = "321";
    await client.addTag(tag);
    await client.removeTag(tag);
    const tags = await client.listTags();
    expect(tags).not.toContain(tag);
  });

  test("cleans up committed note tags after sync", async ({ client, sdk }) => {
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
      sdk.u64(10000000),
      sdk.AuthScheme.AuthRpoFalcon512
    );

    // Mint a note (adds a tag with sourceNoteId for the output note)
    const mintRequest = await client.newMintTransactionRequest(
      wallet.id(),
      faucet.id(),
      sdk.NoteType.Private,
      sdk.u64(1000)
    );
    const mintResult = await executeAndApplyTransaction(
      client,
      sdk,
      faucet.id(),
      mintRequest
    );

    // After applying locally, a note-source tag exists
    const tagsAfterMint = await client.listTags();

    // Commit the block and sync so the transaction is no longer uncommitted
    await client.proveBlock();
    await client.syncState();

    const tagsAfterSync = await client.listTags();

    expect(tagsAfterSync.length).toBeLessThan(tagsAfterMint.length);
  });
});
