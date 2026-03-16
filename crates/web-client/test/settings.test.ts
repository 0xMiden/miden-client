// @ts-nocheck
import { test, expect } from "./test-setup";

test.describe("settings tests", () => {
  test("set and get setting", async ({ client }) => {
    const testValue = [1, 2, 3, 4];
    await client.setSetting("test", testValue);
    const value = await client.getSetting("test");
    expect(JSON.stringify(value)).toEqual(JSON.stringify(testValue));
  });

  test("set and list settings", async ({ client }) => {
    const testKey = "test";
    await client.setSetting(testKey, [1, 2, 3, 4]);
    const keys = await client.listSettingKeys();
    expect(keys.includes(testKey)).toBe(true);
  });

  test("remove setting", async ({ client }) => {
    const testValue = [5, 6, 7, 8];
    await client.setSetting("test", testValue);
    await client.removeSetting("test");

    const resultAfterDelete = await client.getSetting("test");
    const listAfterDelete = await client.listSettingKeys();

    expect(resultAfterDelete).toBeUndefined();
    expect(listAfterDelete.includes("test")).toBe(false);
  });
});
