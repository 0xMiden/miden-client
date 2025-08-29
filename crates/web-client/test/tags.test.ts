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
});
