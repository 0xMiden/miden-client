import { expect, Page } from "@playwright/test";
import test from "./playwright.global.setup";
import { AddressInterface, AccountId, Address, NetworkId } from "../js";
const instanceAddress = async ({
  page,
  accountId,
  _interface,
}: {
  page: typeof Page,
  accountId?: typeof AccountId,
  _interface: typeof AddressInterface,
}) => {
  return await page.evaluate(
    async ({ accountId, _interface}) => {
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
      const address = new window.Address(_accountId, _interface);
      return {

      }
    },
    { accountId, _interface}
  );
};

const instanceAddressTestBech32 = async (page: Page, bech32Prefix: string) => {
  return await page.evaluate(
    async (bech32Prefix) => {
      const client = window.client;
      const newAccount = await client.newWallet(
        window.AccountStorageMode.private(),
        true
      );
      const address = new window.Address(newAccount.id(), "BasicWallet");
      return address.toBech32(bech32Prefix);
    }, bech32Prefix
  );
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

  test("Fail to instance address with something that's not an accound id", async ({
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

  test("Instance address with proper interface", async ({ page }) => {
    await expect(
      instanceAddress({
        page,
        _interface: "Unspecified",
      })
    ).resolves.toBeTruthy();
  });

});

test.describe("Bech32 tests", () => {
  test("bech32 fails with non-valid-prefix", async ({ page }) => {
    await expect(instanceAddressTestBech32(page, "non-valid-prefix")).rejects.toThrow()
  });
  test("bech32 succeeds with mainnet prefix", async ({ page }) => {
    await expect(instanceAddressTestBech32(page, "mm")).resolves.toHaveLength(38);
  });

  test("bech32 succeeds with testnet prefix", async ({ page }) => {
    await expect(instanceAddressTestBech32(page, "mtst")).resolves.toHaveLength(40);
  });

  test("bech32 succeeds with dev prefix", async ({ page }) => {
    await expect(instanceAddressTestBech32(page, "mdev")).resolves.toHaveLength(40);
  });
})