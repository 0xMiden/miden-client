import { expect } from "@playwright/test";
import test from "./playwright.global.setup";

test.describe("settings tests", () => {
  test("set and get setting", async ({ page }) => {
    const isValid = await page.evaluate(async () => {
      const client = window.client;
      const testValue: number[] = [1, 2, 3, 4];
      await client.setValue("test", testValue);

      const value = await client.getValue("test");

      return JSON.stringify(value) === JSON.stringify(testValue);
    });
    expect(isValid).toEqual(true);
  });

  test("set and list settings", async ({ page }) => {
    const isValid = await page.evaluate(async () => {
      const client = window.client;
      const testKey: string = "test";
      await client.setValue(testKey, [1, 2, 3, 4]);

      const keys = await client.listKeys();

      return JSON.stringify(keys) === JSON.stringify([testKey]);
    });
    expect(isValid).toEqual(true);
  });

  test("remove setting", async ({ page }) => {
    const isValid = await page.evaluate(async () => {
      const client = window.client;
      const testValue: number[] = [5, 6, 7, 8];
      await client.setValue("test", testValue);
      await client.removeValue("test");

      const resultAfterDelete = await client.getValue("test");
      const listAfterDelete = await client.listKeys();

      return resultAfterDelete === undefined && listAfterDelete.length === 0;
    });
    expect(isValid).toEqual(true);
  });
});
