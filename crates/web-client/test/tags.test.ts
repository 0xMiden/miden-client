// @ts-nocheck
import { test, expect } from "./test-setup";

// ADD_TAG TESTS
// =======================================================================================================

test.describe("add_tag tests", () => {
  test("adds a tag to the system", async ({ run }) => {
    const result = await run(async ({ client }) => {
      const tag = "123";
      await client.addTag(tag);
      const tags = await client.listTags();
      return { tags, tag };
    });
    expect(result.tags).toContain(result.tag);
  });
});

// REMOVE_TAG TESTS
// =======================================================================================================

test.describe("remove_tag tests", () => {
  test("removes a tag from the system", async ({ run }) => {
    const result = await run(async ({ client }) => {
      const tag = "321";
      await client.addTag(tag);
      await client.removeTag(tag);
      const tags = await client.listTags();
      return { tags, tag };
    });
    expect(result.tags).not.toContain(result.tag);
  });

  test("cleans up committed note tags after sync", async ({ run }) => {
    const result = await run(async ({ client, sdk, helpers }) => {
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
      await helpers.executeAndApplyTransaction(faucet.id(), mintRequest);

      // After applying locally, a note-source tag exists
      const tagsAfterMint = await client.listTags();

      // Commit the block and sync so the transaction is no longer uncommitted
      await client.proveBlock();
      await client.syncState();

      const tagsAfterSync = await client.listTags();

      return {
        tagsAfterMintLength: tagsAfterMint.length,
        tagsAfterSyncLength: tagsAfterSync.length,
      };
    });
    expect(result.tagsAfterSyncLength).toBeLessThan(result.tagsAfterMintLength);
  });
});
