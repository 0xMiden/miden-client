import { expect, Page } from "@playwright/test";
import test from "./playwright.global.setup";
import { AddressInterface, AccountId } from "../js";
const instanceAddress = async ({
  page,
  accountId,
  _interface,
}: {
  page: Page;
  accountId?: typeof AccountId;
  _interface: typeof AddressInterface;
}) => {
  return await page.evaluate(
    async ({ accountId, _interface }) => {
      let _accountId;
      const client = window.client;
      if (accountId) {
        _accountId = accountId;
      } else {
        const newAccount = await client.newWallet(
          window.AccountStorageMode.private(),
          true,
          0
        );
        _accountId = newAccount.id();
      }
      const address = window.Address.fromAccountId(_accountId, _interface);
      return address.interface();
    },
    { accountId, _interface }
  );
};

const instanceNewAddressBech32 = async (page: Page, networkId: string) => {
  return await page.evaluate(async (bech32Prefix) => {
    const client = window.client;
    const parsedNetworkId = window.helpers.parseNetworkId(bech32Prefix);
    const newAccount = await client.newWallet(
      window.AccountStorageMode.private(),
      true,
      0
    );
    const address = window.Address.fromAccountId(
      newAccount.id(),
      "BasicWallet"
    );
    return address.toBech32(parsedNetworkId);
  }, networkId);
};

const instanceAddressFromBech32 = async (
  page: Page,
  bech32EncodedAddress: string
) => {
  return await page.evaluate(async (bech32EncodedAddress) => {
    const parsedNetworkId = window.helpers.parseNetworkId("mtst");
    const address = window.Address.fromBech32(bech32EncodedAddress);
    return address.toBech32(parsedNetworkId) === bech32EncodedAddress;
  }, bech32EncodedAddress);
};

const instanceAddressTestNoteTag = async (page: Page) => {
  return await page.evaluate(async () => {
    const client = window.client;
    const newAccount = await client.newWallet(
      window.AccountStorageMode.private(),
      true,
      0
    );
    const address = window.Address.fromAccountId(
      newAccount.id(),
      "BasicWallet"
    );
    return address.toNoteTag().asU32();
  });
};

test.describe("Address instantiation tests", () => {
  test("Fail to instance address with wrong interface", async ({ page }) => {
    await expect(
      instanceAddress({
        page,
        _interface: "Does not exist",
      })
    ).rejects.toThrow();
  });

  test("Fail to instance address with something that's not an account id", async ({
    page,
  }) => {
    await expect(
      instanceAddress({
        page,
        accountId: "notAnAccountId",
        _interface: "BasicWallet",
      })
    ).rejects.toThrow();
  });

  test("Instance address with proper interface and read it", async ({
    page,
  }) => {
    await expect(
      instanceAddress({
        page,
        _interface: "BasicWallet",
      })
    ).resolves.toBe("BasicWallet");
  });
});

test.describe("Bech32 tests", () => {
  test("to bech32 fails with non-valid-prefix", async ({ page }) => {
    await expect(
      instanceNewAddressBech32(page, "non-valid-prefix")
    ).rejects.toThrow();
  });
  test("encoding from bech32 and going back results in the same address", async ({
    page,
  }) => {
    const expectedBech32 = await instanceNewAddressBech32(page, "mtst");
    await expect(instanceAddressFromBech32(page, expectedBech32)).resolves.toBe(
      true
    );
  });
  test("bech32 succeeds with mainnet prefix", async ({ page }) => {
    await expect(instanceNewAddressBech32(page, "mm")).resolves.toHaveLength(
      47
    );
  });

  test("bech32 succeeds with testnet prefix", async ({ page }) => {
    await expect(instanceNewAddressBech32(page, "mtst")).resolves.toHaveLength(
      49
    );
  });

  test("bech32 succeeds with dev prefix", async ({ page }) => {
    await expect(instanceNewAddressBech32(page, "mdev")).resolves.toHaveLength(
      49
    );
  });

  test("bech32 succeeds with custom prefix", async ({ page }) => {
    await expect(instanceNewAddressBech32(page, "cstm")).resolves.toHaveLength(
      49
    );
  });

  test("fromBech32 returns correct account id", async ({ page }) => {
    const success = await page.evaluate(async () => {
      const newAccount = await window.client.newWallet(
        window.AccountStorageMode.private(),
        true,
        0
      );
      const accountId = newAccount.id();
      const asBech32 = accountId.toBech32(
        window.NetworkId.mainnet(),
        window.AccountInterface.BasicWallet
      );
      const fromBech32 = window.AccountId.fromBech32(asBech32).toString();
      return accountId == fromBech32;
    });
    expect(success).toBe(true);
  });
});

test.describe("Note tag tests", () => {
  test("note tag is returned and read", async ({ page }) => {
    await expect(instanceAddressTestNoteTag(page)).resolves.toBeTruthy();
  });
});

// ADDRESS INSERTION & DELETION TESTS
// =======================================================================================================

const instanceAddressRemoveThenInsert = async (page: Page) => {
  return await page.evaluate(async () => {
    const client = window.client;
    const newAccount = await client.newWallet(
      window.AccountStorageMode.private(),
      true,
      0
    );
    const accountId = newAccount.id().toString();
    const address = window.Address.fromAccountId(newAccount.id(), null);

    // First we remove the address tracked by default
    await client.removeAccountAddress(newAccount.id(), address);

    // Then we add it again
    await client.insertAccountAddress(newAccount.id(), address);

    const store = await client.exportStore();
    const parsedStore = JSON.parse(store);
    const retrievedAddressRecord = parsedStore.addresses[0];
    const retrievedAddress = window.Address.deserialize(
      retrievedAddressRecord.address
    );
    const retrievedId = retrievedAddressRecord.id;

    return {
      accountId: accountId,
      address: address.toBech32(window.NetworkId.testnet()),
      retrievedAccountId: retrievedId,
      retrievedAddress: retrievedAddress.toBech32(window.NetworkId.testnet()),
    };
  });
};

test.describe("Address insertion & deletion tests", () => {
  test("address can be removed and then re-inserted", async ({ page }) => {
    const { accountId, address, retrievedAccountId, retrievedAddress } =
      await instanceAddressRemoveThenInsert(page);
    expect(retrievedAccountId).toBe(accountId);
    expect(address).toBe(retrievedAddress);
  });
});
