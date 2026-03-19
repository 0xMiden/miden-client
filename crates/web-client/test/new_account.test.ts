// @ts-nocheck
import { test, expect } from "./test-setup";

// new_wallet tests
// =======================================================================================================

test.describe("new_wallet tests", () => {
  test("creates a new private, immutable wallet", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
      const newWallet = await client.newWallet(
        sdk.AccountStorageMode.private(),
        false,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      return {
        id: newWallet.id().toString(),
        nonce: newWallet.nonce().toString(),
        vaultRoot: newWallet.vault().root().toHex(),
        storageCommitment: newWallet.storage().commitment().toHex(),
        codeCommitment: newWallet.code().commitment().toHex(),
        isFaucet: newWallet.isFaucet(),
        isRegularAccount: newWallet.isRegularAccount(),
        isUpdatable: newWallet.isUpdatable(),
        isPublic: newWallet.isPublic(),
        isPrivate: newWallet.isPrivate(),
        isNetwork: newWallet.isNetwork(),
        idIsPublic: newWallet.id().isPublic(),
        idIsPrivate: newWallet.id().isPrivate(),
        idIsNetwork: newWallet.id().isNetwork(),
        isNew: newWallet.isNew(),
      };
    });
    expect(result.id.startsWith("0x")).toBe(true);
    expect(result.nonce).toEqual("0");
    expect(result.vaultRoot.startsWith("0x")).toBe(true);
    expect(result.storageCommitment.startsWith("0x")).toBe(true);
    expect(result.codeCommitment.startsWith("0x")).toBe(true);
    expect(result.isFaucet).toEqual(false);
    expect(result.isRegularAccount).toEqual(true);
    expect(result.isUpdatable).toEqual(false);
    expect(result.isPublic).toEqual(false);
    expect(result.isPrivate).toEqual(true);
    expect(result.isNetwork).toEqual(false);
    expect(result.idIsPublic).toEqual(false);
    expect(result.idIsPrivate).toEqual(true);
    expect(result.idIsNetwork).toEqual(false);
    expect(result.isNew).toEqual(true);
  });

  test("creates a new public, immutable wallet", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
      const newWallet = await client.newWallet(
        sdk.AccountStorageMode.public(),
        false,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      return {
        id: newWallet.id().toString(),
        nonce: newWallet.nonce().toString(),
        vaultRoot: newWallet.vault().root().toHex(),
        storageCommitment: newWallet.storage().commitment().toHex(),
        codeCommitment: newWallet.code().commitment().toHex(),
        isFaucet: newWallet.isFaucet(),
        isRegularAccount: newWallet.isRegularAccount(),
        isUpdatable: newWallet.isUpdatable(),
        isPublic: newWallet.isPublic(),
        isPrivate: newWallet.isPrivate(),
        isNetwork: newWallet.isNetwork(),
        idIsPublic: newWallet.id().isPublic(),
        idIsPrivate: newWallet.id().isPrivate(),
        idIsNetwork: newWallet.id().isNetwork(),
        isNew: newWallet.isNew(),
      };
    });
    expect(result.id.startsWith("0x")).toBe(true);
    expect(result.nonce).toEqual("0");
    expect(result.vaultRoot.startsWith("0x")).toBe(true);
    expect(result.storageCommitment.startsWith("0x")).toBe(true);
    expect(result.codeCommitment.startsWith("0x")).toBe(true);
    expect(result.isFaucet).toEqual(false);
    expect(result.isRegularAccount).toEqual(true);
    expect(result.isUpdatable).toEqual(false);
    expect(result.isPublic).toEqual(true);
    expect(result.isPrivate).toEqual(false);
    expect(result.isNetwork).toEqual(false);
    expect(result.idIsPublic).toEqual(true);
    expect(result.idIsPrivate).toEqual(false);
    expect(result.idIsNetwork).toEqual(false);
    expect(result.isNew).toEqual(true);
  });

  test("creates a new private, mutable wallet", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
      const newWallet = await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      return {
        id: newWallet.id().toString(),
        nonce: newWallet.nonce().toString(),
        vaultRoot: newWallet.vault().root().toHex(),
        storageCommitment: newWallet.storage().commitment().toHex(),
        codeCommitment: newWallet.code().commitment().toHex(),
        isFaucet: newWallet.isFaucet(),
        isRegularAccount: newWallet.isRegularAccount(),
        isUpdatable: newWallet.isUpdatable(),
        isPublic: newWallet.isPublic(),
        isPrivate: newWallet.isPrivate(),
        isNetwork: newWallet.isNetwork(),
        idIsPublic: newWallet.id().isPublic(),
        idIsPrivate: newWallet.id().isPrivate(),
        idIsNetwork: newWallet.id().isNetwork(),
        isNew: newWallet.isNew(),
      };
    });
    expect(result.id.startsWith("0x")).toBe(true);
    expect(result.nonce).toEqual("0");
    expect(result.vaultRoot.startsWith("0x")).toBe(true);
    expect(result.storageCommitment.startsWith("0x")).toBe(true);
    expect(result.codeCommitment.startsWith("0x")).toBe(true);
    expect(result.isFaucet).toEqual(false);
    expect(result.isRegularAccount).toEqual(true);
    expect(result.isUpdatable).toEqual(true);
    expect(result.isPublic).toEqual(false);
    expect(result.isPrivate).toEqual(true);
    expect(result.isNetwork).toEqual(false);
    expect(result.idIsPublic).toEqual(false);
    expect(result.idIsPrivate).toEqual(true);
    expect(result.idIsNetwork).toEqual(false);
    expect(result.isNew).toEqual(true);
  });

  test("creates a new public, mutable wallet", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
      const newWallet = await client.newWallet(
        sdk.AccountStorageMode.public(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      return {
        id: newWallet.id().toString(),
        nonce: newWallet.nonce().toString(),
        vaultRoot: newWallet.vault().root().toHex(),
        storageCommitment: newWallet.storage().commitment().toHex(),
        codeCommitment: newWallet.code().commitment().toHex(),
        isFaucet: newWallet.isFaucet(),
        isRegularAccount: newWallet.isRegularAccount(),
        isUpdatable: newWallet.isUpdatable(),
        isPublic: newWallet.isPublic(),
        isPrivate: newWallet.isPrivate(),
        isNetwork: newWallet.isNetwork(),
        idIsPublic: newWallet.id().isPublic(),
        idIsPrivate: newWallet.id().isPrivate(),
        idIsNetwork: newWallet.id().isNetwork(),
        isNew: newWallet.isNew(),
      };
    });
    expect(result.id.startsWith("0x")).toBe(true);
    expect(result.nonce).toEqual("0");
    expect(result.vaultRoot.startsWith("0x")).toBe(true);
    expect(result.storageCommitment.startsWith("0x")).toBe(true);
    expect(result.codeCommitment.startsWith("0x")).toBe(true);
    expect(result.isFaucet).toEqual(false);
    expect(result.isRegularAccount).toEqual(true);
    expect(result.isUpdatable).toEqual(true);
    expect(result.isPublic).toEqual(true);
    expect(result.isPrivate).toEqual(false);
    expect(result.isNetwork).toEqual(false);
    expect(result.idIsPublic).toEqual(true);
    expect(result.idIsPrivate).toEqual(false);
    expect(result.idIsNetwork).toEqual(false);
    expect(result.isNew).toEqual(true);
  });
});

// new_faucet tests
// =======================================================================================================
test.describe("new_faucet tests", () => {
  test("creates a new private, fungible faucet", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
      const newFaucet = await client.newFaucet(
        sdk.AccountStorageMode.private(),
        false,
        "DAG",
        8,
        sdk.u64(10000000),
        sdk.AuthScheme.AuthRpoFalcon512
      );
      return {
        id: newFaucet.id().toString(),
        nonce: newFaucet.nonce().toString(),
        vaultRoot: newFaucet.vault().root().toHex(),
        storageCommitment: newFaucet.storage().commitment().toHex(),
        codeCommitment: newFaucet.code().commitment().toHex(),
        isFaucet: newFaucet.isFaucet(),
        isRegularAccount: newFaucet.isRegularAccount(),
        isUpdatable: newFaucet.isUpdatable(),
        isPublic: newFaucet.isPublic(),
        isPrivate: newFaucet.isPrivate(),
        isNetwork: newFaucet.isNetwork(),
        idIsPublic: newFaucet.id().isPublic(),
        idIsPrivate: newFaucet.id().isPrivate(),
        idIsNetwork: newFaucet.id().isNetwork(),
        isNew: newFaucet.isNew(),
      };
    });
    expect(result.id.startsWith("0x")).toBe(true);
    expect(result.nonce).toEqual("0");
    expect(result.vaultRoot.startsWith("0x")).toBe(true);
    expect(result.storageCommitment.startsWith("0x")).toBe(true);
    expect(result.codeCommitment.startsWith("0x")).toBe(true);
    expect(result.isFaucet).toEqual(true);
    expect(result.isRegularAccount).toEqual(false);
    expect(result.isUpdatable).toEqual(false);
    expect(result.isPublic).toEqual(false);
    expect(result.isPrivate).toEqual(true);
    expect(result.isNetwork).toEqual(false);
    expect(result.idIsPublic).toEqual(false);
    expect(result.idIsPrivate).toEqual(true);
    expect(result.idIsNetwork).toEqual(false);
    expect(result.isNew).toEqual(true);
  });

  test("creates a new public, fungible faucet", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
      const newFaucet = await client.newFaucet(
        sdk.AccountStorageMode.public(),
        false,
        "DAG",
        8,
        sdk.u64(10000000),
        sdk.AuthScheme.AuthRpoFalcon512
      );
      return {
        id: newFaucet.id().toString(),
        nonce: newFaucet.nonce().toString(),
        vaultRoot: newFaucet.vault().root().toHex(),
        storageCommitment: newFaucet.storage().commitment().toHex(),
        codeCommitment: newFaucet.code().commitment().toHex(),
        isFaucet: newFaucet.isFaucet(),
        isRegularAccount: newFaucet.isRegularAccount(),
        isUpdatable: newFaucet.isUpdatable(),
        isPublic: newFaucet.isPublic(),
        isPrivate: newFaucet.isPrivate(),
        isNetwork: newFaucet.isNetwork(),
        idIsPublic: newFaucet.id().isPublic(),
        idIsPrivate: newFaucet.id().isPrivate(),
        idIsNetwork: newFaucet.id().isNetwork(),
        isNew: newFaucet.isNew(),
      };
    });
    expect(result.id.startsWith("0x")).toBe(true);
    expect(result.nonce).toEqual("0");
    expect(result.vaultRoot.startsWith("0x")).toBe(true);
    expect(result.storageCommitment.startsWith("0x")).toBe(true);
    expect(result.codeCommitment.startsWith("0x")).toBe(true);
    expect(result.isFaucet).toEqual(true);
    expect(result.isRegularAccount).toEqual(false);
    expect(result.isUpdatable).toEqual(false);
    expect(result.isPublic).toEqual(true);
    expect(result.isPrivate).toEqual(false);
    expect(result.isNetwork).toEqual(false);
    expect(result.idIsPublic).toEqual(true);
    expect(result.idIsPrivate).toEqual(false);
    expect(result.idIsNetwork).toEqual(false);
    expect(result.isNew).toEqual(true);
  });

  test("throws an error when attempting to create a non-fungible faucet", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk }) => {
      try {
        await client.newFaucet(
          sdk.AccountStorageMode.public(),
          true,
          "DAG",
          8,
          sdk.u64(10000000),
          sdk.AuthScheme.AuthRpoFalcon512
        );
        return { threw: false, errorMessage: "" };
      } catch (e) {
        return { threw: true, errorMessage: e.message || String(e) };
      }
    });
    expect(result.threw).toBe(true);
    expect(result.errorMessage).toContain(
      "Non-fungible faucets are not supported yet"
    );
  });

  test("throws an error when attempting to create a faucet with an invalid token symbol", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk }) => {
      try {
        await client.newFaucet(
          sdk.AccountStorageMode.public(),
          false,
          "INVALID_TOKEN",
          8,
          sdk.u64(10000000),
          sdk.AuthScheme.AuthRpoFalcon512
        );
        return { threw: false, errorMessage: "" };
      } catch (e) {
        return { threw: true, errorMessage: e.message || String(e) };
      }
    });
    expect(result.threw).toBe(true);
    expect(result.errorMessage).toContain(
      "token symbol should have length between 1 and 12 characters, but 13 was provided"
    );
  });
});

// AccountStorage.getMapEntries tests
// =======================================================================================================

test.describe("AccountStorage.getMapEntries tests", () => {
  test("returns undefined for invalid slot names", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
      const NON_MAP_SLOT_NAME =
        "miden::standards::auth::rpo_falcon512::public_key";
      const MISSING_SLOT_NAME =
        "miden::testing::account_storage_tests::missing";

      const account = await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );

      const accountRecord = await client.getAccount(account.id());

      const storage = accountRecord.storage();

      const nonMapResult = storage.getMapEntries(NON_MAP_SLOT_NAME);
      const missingSlotResult = storage.getMapEntries(MISSING_SLOT_NAME);

      return {
        accountRecordDefined: accountRecord !== undefined,
        nonMapResultUndefined: nonMapResult === undefined,
        missingSlotResultUndefined: missingSlotResult === undefined,
      };
    });
    expect(result.accountRecordDefined).toBe(true);
    expect(result.nonMapResultUndefined).toBe(true);
    expect(result.missingSlotResultUndefined).toBe(true);
  });
});
