import { expect } from "chai";
import test from "./playwright.global.setup";
import { Page } from "@playwright/test";

// ADD_TAG TESTS
// =======================================================================================================

interface AddTagSuccessResult {
  tag: string;
  tags: string[];
}

export const addTag = async (
  testingPage: Page,
  tag: string
): Promise<AddTagSuccessResult> => {
  return await testingPage.evaluate(async (tag) => {
    const client = window.client;
    await client.addTag(tag);
    const tags = await client.listTags();

    return {
      tag: tag,
      tags: tags,
    };
  }, tag);
};

test.describe("add_tag tests", () => {
  test("adds a tag to the system", async ({ page }) => {
    const tag = "123";
    const result = await addTag(page, tag);

    expect(result.tags).to.include(tag);
  });
});

// REMOVE_TAG TESTS
// =======================================================================================================

interface RemoveTagSuccessResult {
  tag: string;
  tags: string[];
}

export const removeTag = async (
  testingPage: Page,
  tag: string
): Promise<RemoveTagSuccessResult> => {
  return await testingPage.evaluate(async (tag) => {
    const client = window.client;
    await client.addTag(tag);
    await client.removeTag(tag);

    const tags = await client.listTags();

    return {
      tag: tag,
      tags: tags,
    };
  }, tag);
};

test.describe("remove_tag tests", () => {
  test("removes a tag from the system", async ({ page }) => {
    const tag = "321";
    const result = await removeTag(page, tag);

    expect(result.tags).to.not.include(tag);
  });

  // When a note is created, the client adds a tag with sourceNoteId to track it.
  // After syncing with the node and learning the note is committed, the client
  // should delete that tag since it no longer needs to track it. This verifies
  // the cleanup works by checking that the tag count decreases after sync.
  //
  // Web-client equivalent of the p2id_transfer tag cleanup assertion in
  // crates/testing/miden-client-tests/src/tests.rs (which covers sqlite-store).
  test("cleans up committed note tags after sync", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = window.client;

      const wallet = await client.newWallet(
        window.AccountStorageMode.private(),
        true,
        0
      );
      const faucet = await client.newFaucet(
        window.AccountStorageMode.private(),
        false,
        "DAG",
        8,
        BigInt(10000000)
      );

      // Mint a note (adds a tag with sourceNoteId for the output note)
      const mintRequest = client.newMintTransactionRequest(
        wallet.id(),
        faucet.id(),
        window.NoteType.Private,
        BigInt(1000)
      );
      const mintResult = await window.helpers.executeAndApplyTransaction(
        faucet.id(),
        mintRequest
      );

      // After applying locally, a note-source tag exists
      const tagsAfterMint = await client.listTags();

      // Wait for transaction to be committed (syncs state, triggering
      // updateCommittedNoteTags which should remove the note-source tag)
      await window.helpers.waitForTransaction(
        mintResult.executedTransaction().id().toHex()
      );

      const tagsAfterSync = await client.listTags();

      return {
        tagsAfterMint: tagsAfterMint.length,
        tagsAfterSync: tagsAfterSync.length,
      };
    });

    expect(result.tagsAfterSync).to.be.lessThan(result.tagsAfterMint);
  });
});
