import { expect, Page } from "@playwright/test";
import test from "./playwright.global.setup";
import { AddressInterface, AccountId, Address, NetworkId } from "../js";
const instanceAddress = async ({
  page,
  accountId,
  _interface,
}: {
  page: typeof Page;
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
          true
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
    const newAccount = await client.newWallet(
      window.AccountStorageMode.private(),
      true
    );
    const address = window.Address.fromAccountId(
      newAccount.id(),
      "BasicWallet"
    );
    return address.toBech32(bech32Prefix);
  }, bech32Prefix);
};

const instanceAddressFromBech32 = async (
  page: Page,
  bech32EncodedAddress: string
) => {
  return await page.evaluate(async (bech32EncodedAddress) => {
    const address = window.Address.fromBech32(bech32EncodedAddress);
    return address.toBech32("mtst") === bech32EncodedAddress;
  }, bech32EncodedAddress);
};

const instanceAddressTestNoteTag = async (page: Page) => {
  return await page.evaluate(async () => {
    const client = window.client;
    const newAccount = await client.newWallet(
      window.AccountStorageMode.private(),
      true
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
        _interface: "Unspecified",
      })
    ).rejects.toThrow();
  });

  test("Instance address with proper interface and read it", async ({
    page,
  }) => {
    await expect(
      instanceAddress({
        page,
        _interface: "Unspecified",
      })
    ).resolves.toBe("Unspecified");
  });
});

test.describe("Bech32 tests", () => {
  test("to bech32 fails with non-valid-prefix", async ({ page }) => {
    await expect(
      instanceAddressTestBech32(page, "non-valid-prefix")
    ).rejects.toThrow();
  });
  test("encoding from bech32 and going back results in the same address", async ({
    page,
  }) => {
    const expectedBech32 = "mtst1qz8dmgd0kl2v3ypkgcjf0ewjmacqzzzq3qw";
    await expect(instanceAddressFromBech32(page, expectedBech32)).resolves.toBe(
      true
    );
  });
  test("bech32 succeeds with mainnet prefix", async ({ page }) => {
    await expect(instanceAddressTestBech32(page, "mm")).resolves.toHaveLength(
      38
    );
  });

  test("bech32 succeeds with testnet prefix", async ({ page }) => {
    await expect(instanceAddressTestBech32(page, "mtst")).resolves.toHaveLength(
      40
    );
  });

  test("bech32 succeeds with dev prefix", async ({ page }) => {
    await expect(instanceAddressTestBech32(page, "mdev")).resolves.toHaveLength(
      40
    );
  });
});

test.describe("Note tag tests", () => {
  test("note tag is returned and read", async ({ page }) => {
    await expect(instanceAddressTestNoteTag(page)).resolves.toBeTruthy();
  });
});
