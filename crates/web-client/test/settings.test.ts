import { expect } from "@playwright/test";
import test from "./playwright.global.setup";

test.describe("settings tests", () => {
    test("set, get and delete setting", async ({ page }) => {
        await page.evaluate(async () => {
            const client = window.client;
            const testValue: Uint8Array = new Uint8Array([1, 2, 3, 4]);
            await client.setSettingValue("test", testValue);

            const result = await client.getSettingValue("test");

            expect(result).toEqual(testValue);

            await client.deleteSettingValue("test");

            const resultAfterDelete = await client.getSettingValue("test");

            expect(resultAfterDelete).toEqual(null);
        });
    });
});
