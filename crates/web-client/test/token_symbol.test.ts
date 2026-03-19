// @ts-nocheck
import { test, expect } from "./test-setup";

test.describe("new token symbol", () => {
  test("creates a new token symbol", async ({ run }) => {
    const result = await run(async ({ sdk }) => {
      const tokenSymbol = new sdk.TokenSymbol("MIDEN");
      return tokenSymbol.toString();
    });
    expect(result).toStrictEqual("MIDEN");
  });

  test("thrown an error when creating a token symbol with an empty string", async ({
    run,
  }) => {
    const result = await run(async ({ sdk }) => {
      try {
        new sdk.TokenSymbol("");
        return { threw: false };
      } catch (e) {
        return { threw: true, message: e.message };
      }
    });
    expect(result.threw).toBe(true);
    expect(result.message).toContain(
      "token symbol should have length between 1 and 12 characters, but 0 was provided"
    );
  });

  test("thrown an error when creating a token symbol with more than 12 characters", async ({
    run,
  }) => {
    const result = await run(async ({ sdk }) => {
      try {
        new sdk.TokenSymbol("MIDENTOKENSSS");
        return { threw: false };
      } catch (e) {
        return { threw: true, message: e.message };
      }
    });
    expect(result.threw).toBe(true);
    expect(result.message).toContain(
      "token symbol should have length between 1 and 12 characters, but 13 was provided"
    );
  });
});
