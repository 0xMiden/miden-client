import { expect } from "chai";
import { testingPage } from "./mocha.global.setup.mjs";
import { TokenSymbol } from "../dist/crates/miden_client_web";
import { create } from "domain";

// ADD_TAG TESTS
// =======================================================================================================

interface createNewTokenSymbolResult {
  symbolAsString: string;
}

export const createNewTokenSymbol = async (
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

describe("new token symbol", () => {
  it("creates a new token symbol", async () => {
    const tokenSymbol = "MIDEN";
    const result = await createNewTokenSymbol(tokenSymbol);

    expect(result.symbolAsString).to.equal(tokenSymbol);
  });

  it("thrown an error when creating a token symbol with an empty string", async () => {
    const tokenSymbol = "";

    await expect(createNewTokenSymbol(tokenSymbol)).to.be.rejectedWith(
      "failed to create token symbol: token symbol should have length between 1 and 6 characters, but 0 was provided"
    );
  });

  it("thrown an error when creating a token symbol with more than 6 characters", async () => {
    const tokenSymbol = "MIDENTOKEN";

    await expect(createNewTokenSymbol(tokenSymbol)).to.be.rejectedWith(
      "failed to create token symbol: token symbol should have length between 1 and 6 characters, but 10 was provided"
    );
  });
});
