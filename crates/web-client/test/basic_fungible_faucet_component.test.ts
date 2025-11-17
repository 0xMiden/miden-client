import test from "./playwright.global.setup";
import { create } from "domain";
import { createNewFaucet, createNewWallet } from "./webClientTestUtils";
import { StorageMode } from "./webClientTestUtils";
import { Page, expect } from "@playwright/test";

// BASIC_FUNGIBLE_FAUCET TESTS
// =======================================================================================================

interface basicFungibleFaucetResult {
  symbol: string;
  decimals: number;
  maxSupply: string;
}

export const getBasicFungibleFaucet = async (
  page: Page,
  storageMode: StorageMode = StorageMode.PUBLIC,
  nonFungible: boolean = false,
  tokenSymbol: string = "DAG",
  decimals: number = 8,
  maxSupply: bigint = BigInt(10000000),
  authSchemeId: number = 0
): Promise<basicFungibleFaucetResult> => {
  return await page.evaluate(
    async ({
      _storageMode,
      _nonFungible,
      _tokenSymbol,
      _decimals,
      _maxSupply,
      _authSchemeId,
    }) => {
      const client = window.client;

      const accountStorageMode =
        window.AccountStorageMode.tryFromStr(_storageMode);

      const newFaucet = await client.newFaucet(
        accountStorageMode,
        _nonFungible,
        _tokenSymbol,
        _decimals,
        _maxSupply,
        _authSchemeId
      );

      const basicFungibleFaucet =
        window.BasicFungibleFaucetComponent.fromAccount(newFaucet);

      const result = {
        symbol: basicFungibleFaucet.symbol().toString(),
        decimals: basicFungibleFaucet.decimals(),
        maxSupply: basicFungibleFaucet.maxSupply().toString(),
      };
      return result;
    },
    {
      _storageMode: storageMode,
      _nonFungible: nonFungible,
      _tokenSymbol: tokenSymbol,
      _decimals: decimals,
      _maxSupply: maxSupply,
      _authSchemeId: authSchemeId,
    }
  );
};

export const createWallet = async (
  page: Page
): Promise<basicFungibleFaucetResult> => {
  return await page.evaluate(async () => {
    const client = window.client;
    const account = await client.newWallet(
      window.AccountStorageMode.tryFromStr("PUBLIC"),
      false,
      0,
      undefined
    );
    const basicFungibleFaucet =
      window.BasicFungibleFaucetComponent.fromAccount(account);
    return {
      symbol: basicFungibleFaucet.symbol().toString(),
      decimals: basicFungibleFaucet.decimals(),
      maxSupply: basicFungibleFaucet.maxSupply().toString(),
    };
  });
};

test.describe("basic fungible faucet", () => {
  test("creates a basic fungible faucet component from an account", async ({
    page,
  }) => {
    const faucet = await getBasicFungibleFaucet(page);

    expect(faucet.symbol).toEqual("DAG");
    expect(faucet.decimals).toEqual(8);
    expect(faucet.maxSupply).toEqual("10000000");
  });

  test("throws an error when creating a basic fungible faucet from a non-faucet account", async ({
    page,
  }) => {
    await expect(createWallet(page)).rejects.toThrow(
      "failed to get basic fungible faucet details from account"
    );
  });
});
