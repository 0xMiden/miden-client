import { expect } from "chai";
import { testingPage } from "./mocha.global.setup.mjs";
import { TokenSymbol } from "../dist/crates/miden_client_web";
import { create } from "domain";
import { createNewFaucet, createNewWallet } from "./webClientTestUtils";
import { StorageMode } from "./webClientTestUtils";

// BASIC_FUNGIBLE_FAUCET TESTS
// =======================================================================================================

interface basicFungibleFaucetResult {
  symbol: string;
  decimals: number;
  maxSupply: string;
}

export const getBasicFungibleFaucet = async (
  storageMode: StorageMode = StorageMode.PUBLIC,
  nonFungible: boolean = false,
  tokenSymbol: string = "DAG",
  decimals: number = 8,
  maxSupply: bigint = BigInt(10000000)
): Promise<basicFungibleFaucetResult> => {
  return await testingPage.evaluate(
    async (_storageMode, _nonFungible, _tokenSymbol, _decimals, _maxSupply) => {
      const client = window.client;

      const accountStorageMode =
        window.AccountStorageMode.tryFromStr(_storageMode);

      const newFaucet = await client.newFaucet(
        accountStorageMode,
        _nonFungible,
        _tokenSymbol,
        _decimals,
        _maxSupply
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
    storageMode,
    nonFungible,
    tokenSymbol,
    decimals,
    maxSupply
  );
};

export const createWallet = async (): Promise<basicFungibleFaucetResult> => {
  return await testingPage.evaluate(async () => {
    const client = window.client;
    const account = await client.newWallet(
      window.AccountStorageMode.tryFromStr("PUBLIC"),
      false,
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

describe("basic fungible faucet", () => {
  it("creates a basic fungible faucet component from an account", async () => {
    const faucet = await getBasicFungibleFaucet();

    expect(faucet.symbol).to.equal("DAG");
    expect(faucet.decimals).to.equal(8);
    expect(faucet.maxSupply).to.equal("10000000");
  });

  it("throws an error when creating a basic fungible faucet from a non-faucet account", async () => {
    await expect(createWallet()).to.be.rejectedWith(
      "failed to get basic fungible faucet details from account"
    );
  });
});
