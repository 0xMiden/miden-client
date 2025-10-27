import { Page } from "@playwright/test";
import test from "./playwright.global.setup";
// import { AccountComponent, Package, StorageMap, StorageSlot, TestUtils } from "../dist/crates/miden_client_web";

export const deserializePackageFromBytes = async (
  testingPage: Page
): Promise<void> => {
  await testingPage.evaluate(async () => {
    const testPackageBytes = window.TestUtils.createMockSerializedPackage();
    window.Package.deserialize(testPackageBytes);
  });
};

export const createAccountComponentFromPackage = async (
  testingPage: Page
): Promise<void> => {
  return await testingPage.evaluate(async () => {
    const testPackageBytes = window.TestUtils.createMockSerializedPackage();
    const deserializedPackage = window.Package.deserialize(testPackageBytes);
    let emptyStorageSlot = window.StorageSlot.emptyValue();
    let storageMap = new window.StorageMap();
    let storageSlotMap = window.StorageSlot.map(storageMap);

    window.AccountComponent.fromPackage(deserializedPackage, [
      emptyStorageSlot,
      storageSlotMap,
    ]);
  });
};

test.describe("package tests", () => {
  test("successfully deserializes a package from bytes", async ({ page }) => {
    await deserializePackageFromBytes(page);
  });

  test("creates an account component from a package and storage slot array", async ({
    page,
  }) => {
    await createAccountComponentFromPackage(page);
  });
});
