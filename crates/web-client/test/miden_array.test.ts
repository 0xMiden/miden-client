import { expect, Page } from "@playwright/test";
import { test as base } from "./playwright.global.setup";
import {
  AddressInterface,
  AccountId,
  Address,
  NetworkId,
  MidenArrays,
} from "../js";

// Return each array import as a string array.
const collectArrayTypes = async ({
  page,
}: {
  page: typeof Page;
}): Promise<Array<string>> => {
  return await page.evaluate(async ({}) => {
    return Object.entries(window.MidenArrays).reduce(
      (arrayTypeNames, [arrayTypeName, _]) => {
        arrayTypeNames.push(arrayTypeName);
        return arrayTypeNames;
      },
      []
    );
  }, {});
};

const instanceEmptyArray = async ({
  page,
  arrayTypeToInstance,
}: {
  page: typeof Page;
  arrayTypeToInstance: string;
}) => {
  return await page.evaluate(
    async ({ arrayTypeToInstance: toInstance }) => {
      try {
        console.log(toInstance);
        const array = new window.MidenArrays[toInstance]();
        console.log(array);
        if (array.length() != 0) {
          throw new Error(
            `Newly created array of type ${toInstance} should be zero`
          );
        }
      } catch (err) {
        throw new Error(
          `Failed to build and/or access miden array of type ${toInstance}: ${err}`
        );
      }
      return true;
    },
    { arrayTypeToInstance }
  );
};

const instanceMixedArray = async ({
  page,
  arrayTypeName,
}: {
  page: typeof Page;
  arrayTypeName: string;
}) => {
  return await page.evaluate(
    async ({ arrayType }) => {
      const element = Symbol("not a miden type");
      const midenArray = new window.MidenArrays[arrayType]();
      midenArray.push(element);
    },
    { arrayType: arrayTypeName }
  );
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
        true,
        0
      );
      accounts[i] = account.id();
    }
    const array = new window.MidenArrays.AccountIdArray(accounts);
    return array.length();
  }, {});
};

const mutateAccountIdArray = async ({ page, index }: { page: typeof Page }) => {
  return await page.evaluate(
    async ({ _index }) => {
      const accountToSet = await window.client.newWallet(
        window.AccountStorageMode.private(),
        true,
        0
      );
      const accounts = await Promise.all(
        Array.from({ length: 10 }, () =>
          window.client.newWallet(window.AccountStorageMode.private(), true)
        )
      );
      const accountIds = accounts.map((account) => account.id());
      const array = new window.MidenArrays.AccountIdArray(accountIds);
      array.replaceAt(_index, accountToSet.id());
      return array.get(_index).toString() == accountToSet.id().toString();
    },
    { _index: index }
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
          true,
          0
        );
        accounts[i] = account.id();
      }
      const array = new window.MidenArrays.AccountIdArray(accounts);
      let cloned = array.get(index);
      cloned = await window.client.newWallet(
        window.AccountStorageMode.private(),
        true,
        0
      );
      let original = array.get(index);
      return cloned !== original;
    },
    { index }
  );
};

const arrayWithSingleAccount = async ({ page }: { page: typeof Page }) => {
  return await page.evaluate(async ({}) => {
    const account = await window.client.newWallet(
      window.AccountStorageMode.private(),
      true,
      0
    );
    const array = new window.MidenArrays.AccountArray([]);

    array.push(account);

    return account;
  }, {});
};

const test = base.extend<{ exposedMidenArrayTypes: string[] }>({
  exposedMidenArrayTypes: async ({ page }, use) => {
    let exposedMidenArrayTypes = await collectArrayTypes({ page });
    await use(exposedMidenArrayTypes);
  },
});

test.describe("Specific array tests (using AccountIdArray)", () => {
  test("Cannot modify array through aliasing", async ({ page }) => {
    const params = {
      page,
      index: Math.floor(Math.random() * 10),
    };
    await expect(arrayReturnsClone(params)).resolves.toBe(true);
  });

  test("Pushing into array does not leave variable as undefined", async ({
    page,
  }) => {
    await expect(arrayWithSingleAccount({ page })).resolves.toBeTruthy();
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
      mutateAccountIdArray({
        page,
        index: 5,
      })
    ).resolves.toBe(true);
  });

  test("OOB array mutate throws", async ({ page }) => {
    const index = Math.ceil(Math.random() * (1 << 30)) + 1;
    const params = { page, index };
    await Promise.all([
      expect(mutateAccountIdArray(params)).rejects.toThrowError(
        /out of bounds access/
      ),
      expect(mutateAccountIdArray(params)).rejects.toThrowError(
        /tried to access at index/
      ),
      expect(mutateAccountIdArray(params)).rejects.toThrowError("0"),
      expect(mutateAccountIdArray(params)).rejects.toThrowError("AccountId"),
    ]);
  });
});

test.describe("Generic array tests (using each exposed array type)", () => {
  test("Instance empty arrays", async ({ page, exposedMidenArrayTypes }) => {
    for (const arrayTypeToInstance of exposedMidenArrayTypes) {
      await test.step(`Empty array ${arrayTypeToInstance}`, async () => {
        await expect(
          instanceEmptyArray({
            page,
            arrayTypeToInstance,
          })
        ).resolves.toBe(true);
      });
    }
  });

  test("Building array of mixed types fails", async ({
    page,
    exposedMidenArrayTypes,
  }) => {
    for (const arrayTypeToInstance of exposedMidenArrayTypes) {
      await test.step(`Mixed typed array of ${arrayTypeToInstance} fails`, async () => {
        await expect(
          instanceMixedArray({ page, arrayTypeToInstance }),
          `Should not be able to build array of type ${arrayTypeToInstance} with mixed types`
        ).rejects.toThrow();
      });
    }
  });
});
