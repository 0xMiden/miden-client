import { expect, Page } from "@playwright/test";
import test from "./playwright.global.setup";
import { TokenSymbol } from "../dist/crates/miden_client_web";
import { create } from "domain";

// ADD_TAG TESTS
// =======================================================================================================

interface createNewTokenSymbolResult {
  symbolAsString: string;
}

export const createNewTokenSymbol = async (
  testingPage: Page,
  symbol: string
): Promise<createNewTokenSymbolResult> => {
  return await testingPage.evaluate(async (symbol) => {
    const tokenSymbol = new window.TokenSymbol(symbol);
    const tokenSymbolAsString = tokenSymbol.toString();

    return {
      symbolAsString: tokenSymbolAsString,
    };
  }, symbol);
};

test.describe("new token symbol", () => {
  test("creates a new token symbol", async ({ page }) => {
    const tokenSymbol = "MIDEN";
    const result = await createNewTokenSymbol(page, tokenSymbol);

    expect(result.symbolAsString).toStrictEqual(tokenSymbol);
  });

  test("thrown an error when creating a token symbol with an empty string", async ({
    page,
  }) => {
    const tokenSymbol = "";

    await expect(createNewTokenSymbol(page, tokenSymbol)).rejects.toThrow(
      "failed to create token symbol: token symbol should have length between 1 and 6 characters, but 0 was provided"
    );
  });

  test("thrown an error when creating a token symbol with more than 6 characters", async ({
    page,
  }) => {
    const tokenSymbol = "MIDENTOKEN";

    await expect(createNewTokenSymbol(page, tokenSymbol)).rejects.toThrow(
      "failed to create token symbol: token symbol should have length between 1 and 6 characters, but 10 was provided"
    );
  });
});
