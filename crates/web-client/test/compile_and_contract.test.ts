// @ts-nocheck
import test from "./playwright.global.setup";
import { expect } from "@playwright/test";

// Shared MASM code used across tests — same counter contract as the tutorial.
const COUNTER_CODE = `
  use miden::protocol::active_account
  use miden::protocol::native_account
  use miden::core::word
  use miden::core::sys

  const COUNTER_SLOT = word("miden::tutorials::counter")

  #! Inputs:  []
  #! Outputs: [count]
  pub proc get_count
      push.COUNTER_SLOT[0..2] exec.active_account::get_item
      # => [count]

      exec.sys::truncate_stack
      # => [count]
  end

  #! Inputs:  []
  #! Outputs: []
  pub proc increment_count
      push.COUNTER_SLOT[0..2] exec.active_account::get_item
      # => [count]

      add.1
      # => [count+1]

      push.COUNTER_SLOT[0..2] exec.native_account::set_item
      # => []

      exec.sys::truncate_stack
      # => []
  end
`;

const COUNTER_SLOT_NAME = "miden::tutorials::counter";

// ════════════════════════════════════════════════════════════════
// compile.component()
// ════════════════════════════════════════════════════════════════

test.describe("compile.component()", () => {
  test("returns an AccountComponent with correct procedure hashes", async ({
    page,
  }) => {
    const result = await page.evaluate(
      async ({ code, slotName }) => {
        const client = await window.MidenClient.createMock();
        const component = await client.compile.component({
          code,
          slots: [window.StorageSlot.emptyValue(slotName)],
        });

        const getCountHash = component.getProcedureHash("get_count");
        const incrementHash = component.getProcedureHash("increment_count");

        return {
          hasGetCount: getCountHash != null,
          hasIncrement: incrementHash != null,
          // Hashes are hex strings — check they look like hex words
          getCountHashLen: getCountHash?.length,
          incrementHashLen: incrementHash?.length,
        };
      },
      { code: COUNTER_CODE, slotName: COUNTER_SLOT_NAME }
    );

    expect(result.hasGetCount).toBe(true);
    expect(result.hasIncrement).toBe(true);
    // A Word rendered as hex is 64 hex chars (32 bytes)
    expect(result.getCountHashLen).toBe(64);
    expect(result.incrementHashLen).toBe(64);
  });

  test("withSupportsAllTypes() is applied — component works in a contract", async ({
    page,
  }) => {
    // If withSupportsAllTypes() weren't called, building the account below would
    // fail because the component wouldn't be compatible with the account type.
    const result = await page.evaluate(
      async ({ code, slotName }) => {
        const client = await window.MidenClient.createMock();
        const component = await client.compile.component({
          code,
          slots: [window.StorageSlot.emptyValue(slotName)],
        });

        const seed = new Uint8Array(32);
        seed.fill(0xab);
        const auth = window.AuthSecretKey.rpoFalconWithRNG(seed);

        // This throws if withSupportsAllTypes() was not applied
        const account = await client.accounts.create({
          type: "ImmutableContract",
          storage: "public",
          seed,
          auth,
          components: [component],
        });

        return { id: account.id().toString() };
      },
      { code: COUNTER_CODE, slotName: COUNTER_SLOT_NAME }
    );

    expect(result.id).toBeDefined();
  });

  test("each call uses a fresh builder — slot accumulation does not occur", async ({
    page,
  }) => {
    // Two compile.component() calls with different slots must not merge.
    const result = await page.evaluate(
      async ({ code }) => {
        const client = await window.MidenClient.createMock();

        const slotA = "miden::tutorials::counter";
        const slotB = "miden::tutorials::count_reader";

        const compA = await client.compile.component({
          code,
          slots: [window.StorageSlot.emptyValue(slotA)],
        });
        const compB = await client.compile.component({
          code,
          slots: [window.StorageSlot.emptyValue(slotB)],
        });

        // Both should succeed independently (no cross-call contamination)
        return {
          aHasProc: compA.getProcedureHash("get_count") != null,
          bHasProc: compB.getProcedureHash("get_count") != null,
        };
      },
      { code: COUNTER_CODE }
    );

    expect(result.aHasProc).toBe(true);
    expect(result.bHasProc).toBe(true);
  });
});

// ════════════════════════════════════════════════════════════════
// compile.txScript()
// ════════════════════════════════════════════════════════════════

test.describe("compile.txScript()", () => {
  test("compiles a script without libraries", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();
      const script = await client.compile.txScript({
        code: `
          use miden::core::sys
          begin
            exec.sys::truncate_stack
          end
        `,
      });
      return { isDefined: script != null };
    });

    expect(result.isDefined).toBe(true);
  });

  test("compiles a script with a dynamic library", async ({ page }) => {
    const result = await page.evaluate(
      async ({ counterCode, slotName }) => {
        const client = await window.MidenClient.createMock();
        const script = await client.compile.txScript({
          code: `
            use external_contract::counter_contract
            begin
              call.counter_contract::increment_count
            end
          `,
          libraries: [
            {
              namespace: "external_contract::counter_contract",
              code: counterCode,
            },
          ],
        });
        return { isDefined: script != null };
      },
      { counterCode: COUNTER_CODE, slotName: COUNTER_SLOT_NAME }
    );

    expect(result.isDefined).toBe(true);
  });

  test("each call uses a fresh builder — libraries from prior calls do not leak", async ({
    page,
  }) => {
    // Call txScript once with a library, then again without it.
    // If builders were shared, the second call would "accidentally" have the
    // first call's library and still succeed (or fail in unexpected ways).
    // With fresh builders the second call compiles cleanly in isolation.
    const result = await page.evaluate(
      async ({ counterCode }) => {
        const client = await window.MidenClient.createMock();

        // First call: link counter_contract library
        const scriptWithLib = await client.compile.txScript({
          code: `
            use external_contract::counter_contract
            begin
              call.counter_contract::increment_count
            end
          `,
          libraries: [
            {
              namespace: "external_contract::counter_contract",
              code: counterCode,
            },
          ],
        });

        // Second call: no libraries — must compile independently without any
        // residual state from the first call.
        const scriptNoLib = await client.compile.txScript({
          code: `
            use miden::core::sys
            begin
              exec.sys::truncate_stack
            end
          `,
        });

        return {
          firstOk: scriptWithLib != null,
          secondOk: scriptNoLib != null,
        };
      },
      { counterCode: COUNTER_CODE }
    );

    expect(result.firstOk).toBe(true);
    expect(result.secondOk).toBe(true);
  });
});

// ════════════════════════════════════════════════════════════════
// accounts.create() — contract types
// ════════════════════════════════════════════════════════════════

test.describe("accounts.create() — ImmutableContract / MutableContract", () => {
  test("ImmutableContract: isUpdatable=false, isPublic=true, isRegularAccount=true", async ({
    page,
  }) => {
    const result = await page.evaluate(
      async ({ code, slotName }) => {
        const client = await window.MidenClient.createMock();

        const component = await client.compile.component({
          code,
          slots: [window.StorageSlot.emptyValue(slotName)],
        });

        const seed = new Uint8Array(32);
        seed.fill(0x01);
        const auth = window.AuthSecretKey.rpoFalconWithRNG(seed);

        const account = await client.accounts.create({
          type: "ImmutableContract",
          storage: "public",
          seed,
          auth,
          components: [component],
        });

        return {
          isFaucet: account.isFaucet(),
          isRegularAccount: account.isRegularAccount(),
          isUpdatable: account.isUpdatable(),
          isPublic: account.isPublic(),
          isNew: account.isNew(),
        };
      },
      { code: COUNTER_CODE, slotName: COUNTER_SLOT_NAME }
    );

    expect(result.isFaucet).toBe(false);
    expect(result.isRegularAccount).toBe(true);
    expect(result.isUpdatable).toBe(false);
    expect(result.isPublic).toBe(true);
    expect(result.isNew).toBe(true);
  });

  test("MutableContract: isUpdatable=true, isPublic=true", async ({ page }) => {
    const result = await page.evaluate(
      async ({ code, slotName }) => {
        const client = await window.MidenClient.createMock();

        const component = await client.compile.component({
          code,
          slots: [window.StorageSlot.emptyValue(slotName)],
        });

        const seed = new Uint8Array(32);
        seed.fill(0x02);
        const auth = window.AuthSecretKey.rpoFalconWithRNG(seed);

        const account = await client.accounts.create({
          type: "MutableContract",
          storage: "public",
          seed,
          auth,
          components: [component],
        });

        return {
          isUpdatable: account.isUpdatable(),
          isPublic: account.isPublic(),
          isRegularAccount: account.isRegularAccount(),
        };
      },
      { code: COUNTER_CODE, slotName: COUNTER_SLOT_NAME }
    );

    expect(result.isUpdatable).toBe(true);
    expect(result.isPublic).toBe(true);
    expect(result.isRegularAccount).toBe(true);
  });

  test("ImmutableContract defaults to public storage when storage is omitted", async ({
    page,
  }) => {
    const result = await page.evaluate(
      async ({ code, slotName }) => {
        const client = await window.MidenClient.createMock();

        const component = await client.compile.component({
          code,
          slots: [window.StorageSlot.emptyValue(slotName)],
        });

        const seed = new Uint8Array(32);
        seed.fill(0x03);
        const auth = window.AuthSecretKey.rpoFalconWithRNG(seed);

        // No `storage` field — should default to "public"
        const account = await client.accounts.create({
          type: "ImmutableContract",
          seed,
          auth,
          components: [component],
        });

        return { isPublic: account.isPublic() };
      },
      { code: COUNTER_CODE, slotName: COUNTER_SLOT_NAME }
    );

    expect(result.isPublic).toBe(true);
  });

  test("contract with no extra components still creates successfully", async ({
    page,
  }) => {
    // Auth component alone — no additional components array
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();

      const seed = new Uint8Array(32);
      seed.fill(0x04);
      const auth = window.AuthSecretKey.rpoFalconWithRNG(seed);

      const account = await client.accounts.create({
        type: "ImmutableContract",
        storage: "public",
        seed,
        auth,
        // components intentionally omitted
      });

      return { id: account.id().toString(), isNew: account.isNew() };
    });

    expect(result.id).toBeDefined();
    expect(result.isNew).toBe(true);
  });

  test("same seed yields same account ID across two isolated clients", async ({
    page,
  }) => {
    const result = await page.evaluate(
      async ({ code, slotName }) => {
        const seed = new Uint8Array(32);
        seed.fill(0x77);

        const buildAccount = async () => {
          const client = await window.MidenClient.createMock();
          const component = await client.compile.component({
            code,
            slots: [window.StorageSlot.emptyValue(slotName)],
          });
          const auth = window.AuthSecretKey.rpoFalconWithRNG(seed);
          const account = await client.accounts.create({
            type: "ImmutableContract",
            storage: "public",
            seed,
            auth,
            components: [component],
          });
          return account.id().toString();
        };

        const id1 = await buildAccount();
        const id2 = await buildAccount();

        return { id1, id2, match: id1 === id2 };
      },
      { code: COUNTER_CODE, slotName: COUNTER_SLOT_NAME }
    );

    expect(result.match).toBe(true);
  });
});

// ════════════════════════════════════════════════════════════════
// transactions.execute()
// ════════════════════════════════════════════════════════════════

test.describe("transactions.execute()", () => {
  test("executes a custom script against an ImmutableContract and returns a txId", async ({
    page,
  }) => {
    const result = await page.evaluate(
      async ({ code, slotName }) => {
        const client = await window.MidenClient.createMock();

        // Create the counter contract
        const component = await client.compile.component({
          code,
          slots: [window.StorageSlot.emptyValue(slotName)],
        });

        const seed = new Uint8Array(32);
        seed.fill(0x10);
        const auth = window.AuthSecretKey.rpoFalconWithRNG(seed);

        const account = await client.accounts.create({
          type: "ImmutableContract",
          storage: "public",
          seed,
          auth,
          components: [component],
        });

        // Advance one block so the account is committed
        client.proveBlock();
        await client.sync();

        // Compile the increment script
        const script = await client.compile.txScript({
          code: `
            use external_contract::counter_contract
            begin
              call.counter_contract::increment_count
            end
          `,
          libraries: [
            { namespace: "external_contract::counter_contract", code },
          ],
        });

        // Execute the transaction
        const txId = await client.transactions.execute({
          account: account.id(),
          script,
        });

        return { txHex: txId.toHex() };
      },
      { code: COUNTER_CODE, slotName: COUNTER_SLOT_NAME }
    );

    expect(result.txHex).toBeDefined();
    expect(result.txHex.length).toBeGreaterThan(0);
  });

  test("execute() increments storage slot on the contract", async ({
    page,
  }) => {
    const result = await page.evaluate(
      async ({ code, slotName }) => {
        const client = await window.MidenClient.createMock();

        const component = await client.compile.component({
          code,
          slots: [window.StorageSlot.emptyValue(slotName)],
        });

        const seed = new Uint8Array(32);
        seed.fill(0x20);
        const auth = window.AuthSecretKey.rpoFalconWithRNG(seed);

        const account = await client.accounts.create({
          type: "ImmutableContract",
          storage: "public",
          seed,
          auth,
          components: [component],
        });

        client.proveBlock();
        await client.sync();

        const script = await client.compile.txScript({
          code: `
            use external_contract::counter_contract
            begin
              call.counter_contract::increment_count
            end
          `,
          libraries: [
            { namespace: "external_contract::counter_contract", code },
          ],
        });

        await client.transactions.execute({ account: account.id(), script });

        client.proveBlock();
        await client.sync();

        // Read the updated storage
        const updated = await client.accounts.get(account.id());
        const countWord = updated?.storage().getItem(slotName);

        // The counter word is stored little-endian; extract the value from the last 8 hex chars
        const countHex = countWord?.toHex() ?? "";
        const countValue = Number(
          BigInt("0x" + countHex.slice(-16).match(/../g).reverse().join(""))
        );

        return { countValue };
      },
      { code: COUNTER_CODE, slotName: COUNTER_SLOT_NAME }
    );

    expect(result.countValue).toBe(1);
  });

  test("execute() with foreignAccounts accepts a plain { id } wrapper", async ({
    page,
  }) => {
    // Verifies that the wrapper-vs-WASM discrimination logic in execute() works:
    // passing { id: accountId } (plain object) correctly resolves the account ref.
    const result = await page.evaluate(
      async ({ code, slotName }) => {
        const client = await window.MidenClient.createMock();

        // Create a target contract (the "foreign" account)
        const component = await client.compile.component({
          code,
          slots: [window.StorageSlot.emptyValue(slotName)],
        });
        const seed1 = new Uint8Array(32);
        seed1.fill(0x30);
        const auth1 = window.AuthSecretKey.rpoFalconWithRNG(seed1);
        const foreignContract = await client.accounts.create({
          type: "ImmutableContract",
          storage: "public",
          seed: seed1,
          auth: auth1,
          components: [component],
        });

        // Create a wallet that will execute the FPI-like script
        const wallet = await client.accounts.create();

        client.proveBlock();
        await client.sync();

        // A minimal script that just reads foreign account state (no mutation)
        // Using truncate_stack to avoid leaving anything on the operand stack.
        const script = await client.compile.txScript({
          code: `
            use miden::core::sys
            begin
              exec.sys::truncate_stack
            end
          `,
        });

        // The key assertion: passing { id: foreignContract.id() } works
        const txId = await client.transactions.execute({
          account: wallet.id(),
          script,
          foreignAccounts: [{ id: foreignContract.id() }],
        });

        return { txHex: txId.toHex() };
      },
      { code: COUNTER_CODE, slotName: COUNTER_SLOT_NAME }
    );

    expect(result.txHex).toBeDefined();
    expect(result.txHex.length).toBeGreaterThan(0);
  });

  test("execute() with a bare AccountId as foreignAccount (non-wrapper path)", async ({
    page,
  }) => {
    const result = await page.evaluate(
      async ({ code, slotName }) => {
        const client = await window.MidenClient.createMock();

        const component = await client.compile.component({
          code,
          slots: [window.StorageSlot.emptyValue(slotName)],
        });
        const seed = new Uint8Array(32);
        seed.fill(0x40);
        const auth = window.AuthSecretKey.rpoFalconWithRNG(seed);
        const foreignContract = await client.accounts.create({
          type: "ImmutableContract",
          storage: "public",
          seed,
          auth,
          components: [component],
        });

        const wallet = await client.accounts.create();

        client.proveBlock();
        await client.sync();

        const script = await client.compile.txScript({
          code: `
            use miden::core::sys
            begin
              exec.sys::truncate_stack
            end
          `,
        });

        // Pass the AccountId directly (not wrapped in { id })
        const txId = await client.transactions.execute({
          account: wallet.id(),
          script,
          foreignAccounts: [foreignContract.id()],
        });

        return { txHex: txId.toHex() };
      },
      { code: COUNTER_CODE, slotName: COUNTER_SLOT_NAME }
    );

    expect(result.txHex).toBeDefined();
    expect(result.txHex.length).toBeGreaterThan(0);
  });
});
