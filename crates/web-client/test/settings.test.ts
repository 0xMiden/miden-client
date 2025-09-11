import { expect } from "@playwright/test";
import test from "./playwright.global.setup";

test.describe("settings tests", () => {
  test("set and get setting", async ({ page }) => {
    const isValid = await page.evaluate(async () => {
      const client = window.client;
      const testValue: number[] = [1, 2, 3, 4];
      await client.setSettingValue("test", testValue);

      const result = await client.getSettingValue("test");

      return (JSON.stringify(result) === JSON.stringify(testValue));
    });
    expect(isValid).toEqual(true);
  });


  test("delete setting", async ({ page }) => {
    const isValid = await page.evaluate(async () => {
      const client = window.client;
      const testValue: number[] = [5, 6, 7, 8];
      await client.setSettingValue("test", testValue);
      await client.deleteSettingValue("test");

      const resultAfterDelete = await client.getSettingValue("test");

      return (resultAfterDelete === undefined);
    });
    expect(isValid).toEqual(true);
  });
});
