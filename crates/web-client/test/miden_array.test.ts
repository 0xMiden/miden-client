import { expect, Page } from "@playwright/test";
import test from "./playwright.global.setup";
import {
  AddressInterface,
  AccountId,
  Address,
  NetworkId,
  AccountIdArray,
  MidenArrays,
} from "../js";

const collectArrayTypes = async ({
  page,
}: {
  page: typeof Page;
}): Promise<Array<string>> => {
  return await page.evaluate(async ({}) => {
    return Object.entries(window.MidenArrays).reduce(
      (arrayTypeNames, [arrayTypeName, _]) => {
        // FIXME: Avoid filtering these before finishing PR.
        if (
          arrayTypeName != "FeltArray" &&
          arrayTypeName != "NoteDetailsArray" &&
          arrayTypeName != "OutputNotesArray" &&
          !arrayTypeName.startsWith("Note") &&
          arrayTypeName != "RecipientArray" &&
          arrayTypeName != "TransactionScriptInputPairArray"
        ) {
          arrayTypeNames.push(arrayTypeName);
        }
        return arrayTypeNames;
      },
      []
    );
  }, {});
};

const instanceEmptyArrays = async ({ page }: { page: typeof Page }) => {
  return await page.evaluate(async ({}) => {
    for (const [arrayName, arrayBuilder] of Object.entries(
      window.MidenArrays
    )) {
      // FIXME: Avoid filtering these before finishing PR.
      if (
        arrayName != "FeltArray" &&
        arrayName != "NoteDetailsArray" &&
        arrayName != "OutputNotesArray" &&
        !arrayName.startsWith("Note") &&
        arrayName != "RecipientArray" &&
        arrayName != "TransactionScriptInputPairArray"
      ) {
        try {
          const array = new window.MidenArrays[arrayName]();
          console.log(array);
          if (array.length() != 0) {
            throw new Error(
              `Newly created array of type ${arrayName} should be zero`
            );
          }
        } catch (err) {
          throw new Error(
            `Failed to build and/or access miden array of type ${arrayName}: ${err}`
          );
        }
      }
    }
    return true;
  }, {});
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
        true
      );
      accounts[i] = account.id();
    }
    const array = new window.AccountIdArray(accounts);
    return array.length();
  }, {});
};

const mutateArrayAtIndex = async ({ page }: { page: typeof Page }) => {
  return await page.evaluate(async ({}) => {
    const accountToSet = await window.client.newWallet(
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
    array.replaceAt(5, accountToSet.id());
    return array.at(5).toString() == accountToSet.id().toString();
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

const arrayWithSingleAccount = async ({ page }: { page: typeof Page }) => {
  return await page.evaluate(async ({}) => {
    const account = await window.client.newWallet(
      window.AccountStorageMode.private(),
      true
    );
    const array = new window.MidenArrays.AccountArray([]);

    array.push(account);

    return account;
  }, {});
};

test.describe("Instance array", () => {
  test("Instance empty arrays", async ({ page }) => {
    await expect(
      instanceEmptyArrays({
        page,
      })
    ).resolves.toBe(true);
  });

  test("Building array of mixed types fails", async ({ page }) => {
    const arrayTypes = await collectArrayTypes({ page });
    await Promise.all(
      arrayTypes.map((arrayTypeName) => {
        expect(
          instanceMixedArray({ page, arrayTypeName }),
          `Should not be able to build array of type ${arrayTypeName} with mixed types`
        ).rejects.toThrow();
      })
    );
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

  test("Pushing into array does not leave variable as undefined", async ({
    page,
  }) => {
    await expect(arrayWithSingleAccount({ page })).resolves.toBeTruthy();
  });
});
