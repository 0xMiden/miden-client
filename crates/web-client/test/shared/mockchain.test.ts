import { test, expect } from "./fixtures.ts";

test.describe("mock chain tests", () => {
  test("mint and consume transaction completes successfully", async ({
    ops,
  }) => {
    const balance = await ops.mockChainMintAndConsume();
    expect(balance).toBe("1000");
  });
});
