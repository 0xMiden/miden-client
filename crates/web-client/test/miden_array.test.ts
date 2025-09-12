import { expect, Page } from "@playwright/test";
import test from "./playwright.global.setup";
import {
  AddressInterface,
  AccountId,
  Address,
  NetworkId,
  AccountIdArray,
} from "../js";

const instanceEmptyAccountArray = async ({ page }: { page: typeof Page }) => {
  return await page.evaluate(async ({}) => {
    const array = new window.AccountIdArray([]);
    return array.length;
  }, {});
};

const instanceAccountArrayFromAccounts = async ({
  page,
}: {
  page: typeof Page;
}) => {
  return await page.evaluate(async ({}) => {
    let accounts = [];
    for (let i = 0; i < 10; i++) {
      const account = await window.client.newWallet(
        window.AccountStorageMode.private(),
        true
      );
      accounts[i] = account.id();
    }
    const array = new window.AccountIdArray(accounts);
    return array.length;
  }, {});
};

const mutateArrayAtIndex = async ({ page }: { page: typeof Page }) => {
  return await page.evaluate(async ({}) => {
    const account_to_set = await window.client.newWallet(
      window.AccountStorageMode.private(),
      true
    );
    const accounts = await Promise.all(
      Array.from({ length: 10 }, () =>
        window.client.newWallet(window.AccountStorageMode.private(), true)
      )
    );
    const account_ids = accounts.map((account) => account.id());
    const array = new window.AccountIdArray(account_ids);
    array.replaceAt(5, account_to_set.id());
    return array.at(5).toString() == account_to_set.id().toString();
  }, {});
};

const outOfBoundsArrayAccess = async ({
  page,
  index,
}: {
  page: typeof Page;
  index: number;
}) => {
  return await page.evaluate(
    async ({ index }) => {
      const array = new window.AccountIdArray([]);
      return array.at(index);
    },
    { index }
  );
};

const outOfBoundsReplace = async ({
  page,
  index,
}: {
  page: typeof Page;
  index: number;
}) => {
  return await page.evaluate(
    async ({ index }) => {
      const wallet = await window.client.newWallet(
        window.AccountStorageMode.private(),
        true
      );
      const accountId = wallet.id();
      const array = new window.AccountIdArray([]);
      return array.replaceAt(index, accountId);
    },
    { index }
  );
};

const arrayReturnsClone = async ({
  page,
  index,
}: {
  page: typeof Page;
  index: number;
}) => {
  return await page.evaluate(
    async ({ index }) => {
      let accounts = [];
      for (let i = 0; i < 10; i++) {
        const account = await window.client.newWallet(
          window.AccountStorageMode.private(),
          true
        );
        accounts[i] = account.id();
      }
      const array = new window.AccountIdArray(accounts);
      let cloned = array.at(index);
      cloned = await window.client.newWallet(
        window.AccountStorageMode.private(),
        true
      );
      let original = array.at(index);
      return cloned !== original;
    },
    { index }
  );
};

test.describe("Instance array", () => {
  test("Instance empty array", async ({ page }) => {
    await expect(
      instanceEmptyAccountArray({
        page,
      })
    ).resolves.toBe(0);
  });

  test("Instance array with 10 account ids ", async ({ page }) => {
    await expect(
      instanceAccountArrayFromAccounts({
        page,
      })
    ).resolves.toBe(10);
  });

  test("Mutate array at index", async ({ page }) => {
    await expect(
      mutateArrayAtIndex({
        page,
      })
    ).resolves.toBe(true);
  });

  test("OOB index throws", async ({ page }) => {
    const index = Math.random() * (1 << 30);
    const params = { page, index };
    await Promise.all([
      expect(outOfBoundsArrayAccess(params)).rejects.toThrowError(
        /out of bounds access/
      ),
      expect(outOfBoundsArrayAccess(params)).rejects.toThrowError(
        /tried to access at index/
      ),
      expect(outOfBoundsArrayAccess(params)).rejects.toThrowError("0"),
      expect(outOfBoundsArrayAccess(params)).rejects.toThrowError("AccountId"),
      expect(outOfBoundsReplace(params)).rejects.toThrowError(
        /out of bounds access/
      ),
      expect(outOfBoundsReplace(params)).rejects.toThrowError(
        /tried to access at index/
      ),
      expect(outOfBoundsReplace(params)).rejects.toThrowError("0"),
      expect(outOfBoundsReplace(params)).rejects.toThrowError("AccountId"),
    ]);
  });
  test("Cannot modify array through aliasing", async ({ page }) => {
    const params = {
      page,
      index: Math.floor(Math.random() * 10),
    };
    await expect(arrayReturnsClone(params)).resolves.toBe(true);
  });
});
