// @ts-nocheck
import { test, expect } from "./test-setup";

test.describe("new token symbol", () => {
  test("creates a new token symbol", async ({ sdk }) => {
    const tokenSymbol = new sdk.TokenSymbol("MIDEN");
    expect(tokenSymbol.toString()).toStrictEqual("MIDEN");
  });

  test("thrown an error when creating a token symbol with an empty string", async ({
    sdk,
  }) => {
    expect(() => new sdk.TokenSymbol("")).toThrow(
      "failed to create token symbol: token symbol should have length between 1 and 12 characters, but 0 was provided"
    );
  });

  test("thrown an error when creating a token symbol with more than 12 characters", async ({
    sdk,
  }) => {
    expect(() => new sdk.TokenSymbol("MIDENTOKENSSS")).toThrow(
      "failed to create token symbol: token symbol should have length between 1 and 12 characters, but 13 was provided"
    );
  });
});
