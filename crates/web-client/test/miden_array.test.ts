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
        const array = new window.MidenArrays[toInstance]();
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
        window.AuthScheme.AuthRpoFalcon512
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
        window.AuthScheme.AuthRpoFalcon512
      );
      const accounts = await Promise.all(
        Array.from({ length: 10 }, () =>
          window.client.newWallet(
            window.AccountStorageMode.private(),
            true,
            window.AuthScheme.AuthRpoFalcon512
          )
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
          window.AuthScheme.AuthRpoFalcon512
        );
        accounts[i] = account.id();
      }
      const array = new window.MidenArrays.AccountIdArray(accounts);
      let cloned = array.get(index);
      cloned = await window.client.newWallet(
        window.AccountStorageMode.private(),
        true,
        window.AuthScheme.AuthRpoFalcon512
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
      window.AuthScheme.AuthRpoFalcon512
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
    try {
      await mutateAccountIdArray({ page, index });
      throw new Error("Expected mutateAccountIdArray to throw");
    } catch (error: any) {
      const message = error.message || String(error);
      expect(message).toMatch(/out of bounds access/);
      expect(message).toMatch(/tried to access at index/);
      expect(message).toContain("0");
      expect(message).toContain("AccountId");
    }
  });
});

test.describe("Constructor preserves input handles (regression for #2122)", () => {
  // The wasm-bindgen-generated `pub fn new(elements: Option<Vec<T>>)`
  // constructor takes each element by value: the Rust-side value is moved
  // out of the caller's JS handle, leaving the JS object's `__wbg_ptr`
  // pointing at a freed slot. Subsequent method calls on the original
  // handle then panic deep inside WASM with the opaque
  // `"null pointer passed to rust"` error.
  //
  // The fix in `js/safe-arrays.js` overrides the generated constructor
  // with a wrapper that builds via `push(&T)` — which borrows + clones —
  // leaving every input handle fully usable afterwards. These tests pin
  // that contract for the four most common element types. They fail on
  // origin/main with the null-pointer panic; they pass with the fix.
  //
  // The original repro from #2122 — `new Note(...) -> new NoteArray([note]) ->
  // note.id()` — is covered by `NoteArray` below.

  test("FeltArray constructor preserves Felt handles", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const felts = [10n, 20n, 30n].map((v) => new window.Felt(v));
      // Pre-fix: this constructor moves each Felt's Rust value out of
      // its JS handle.
      const _array = new window.FeltArray(felts);
      // Pre-fix: every one of these `asInt()` calls panics.
      return felts.map((f) => f.asInt().toString());
    });
    expect(result).toEqual(["10", "20", "30"]);
  });

  test("AccountIdArray constructor preserves AccountId handles", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const ids = [];
      for (let i = 0; i < 3; i++) {
        const wallet = await window.client.newWallet(
          window.AccountStorageMode.private(),
          true,
          window.AuthScheme.AuthRpoFalcon512
        );
        ids.push(wallet.id());
      }
      // Pre-fix: this constructor moves each AccountId's Rust value out.
      const _array = new window.AccountIdArray(ids);
      // Pre-fix: every `toString()` panics.
      return ids.map((id) => id.toString());
    });
    expect(result).toHaveLength(3);
    for (const idStr of result) {
      expect(typeof idStr).toBe("string");
      expect(idStr.length).toBeGreaterThan(0);
    }
  });

  test("AccountArray constructor preserves Account handles", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const account = await window.client.newWallet(
        window.AccountStorageMode.private(),
        true,
        window.AuthScheme.AuthRpoFalcon512
      );
      // Pre-fix: this constructor consumes the Account.
      const _array = new window.AccountArray([account]);
      // Pre-fix: `account.id()` reads through the dangling __wbg_ptr and
      // panics. After the fix, the handle is still live.
      return account.id().toString();
    });
    expect(typeof result).toBe("string");
    expect(result.length).toBeGreaterThan(0);
  });

  test("NoteArray constructor preserves Note handle (the original repro)", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const sender = await window.client.newWallet(
        window.AccountStorageMode.private(),
        true,
        window.AuthScheme.AuthRpoFalcon512
      );
      const receiver = await window.client.newWallet(
        window.AccountStorageMode.private(),
        true,
        window.AuthScheme.AuthRpoFalcon512
      );
      const noteAssets = new window.NoteAssets([]);
      const note = window.Note.createP2IDNote(
        sender.id(),
        receiver.id(),
        noteAssets,
        window.NoteType.Public,
        new window.NoteAttachment()
      );
      // Pre-fix: `new NoteArray([note])` moved note's Rust value out.
      const _array = new window.NoteArray([note]);
      // Pre-fix: `note.id()` panics. This is the exact reproducer from
      // #2122 — the call surfaced in the original Battleship/wallet
      // investigation as a phantom wallet bug.
      return note.id().toString();
    });
    expect(typeof result).toBe("string");
    expect(result.length).toBeGreaterThan(0);
  });

  test("replaceAt preserves the input handle", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const wallets = [];
      for (let i = 0; i < 3; i++) {
        wallets.push(
          await window.client.newWallet(
            window.AccountStorageMode.private(),
            true,
            window.AuthScheme.AuthRpoFalcon512
          )
        );
      }
      const ids = wallets.map((w) => w.id());
      const array = new window.AccountIdArray(ids);

      // The new id we're going to plug in. Pre-fix, `replaceAt` took
      // its element by value, moving the Rust value out of `replacement`'s
      // JS handle. The Rust-side fix at miden_array.rs (`replaceAt` now
      // takes `&T`) closes that.
      const replacementWallet = await window.client.newWallet(
        window.AccountStorageMode.private(),
        true,
        window.AuthScheme.AuthRpoFalcon512
      );
      const replacement = replacementWallet.id();
      array.replaceAt(1, replacement);

      // Pre-fix: this `toString()` would panic.
      return replacement.toString();
    });
    expect(typeof result).toBe("string");
    expect(result.length).toBeGreaterThan(0);
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
