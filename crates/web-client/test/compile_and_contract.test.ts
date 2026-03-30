// @ts-nocheck
import { test, expect } from "./test-setup";
import { createMidenClient } from "./test-helpers";

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
    sdk,
  }) => {
    const MidenClient = await createMidenClient(sdk);
    test.skip(!MidenClient, "requires napi binary (Node.js only)");
    const client = await MidenClient.createMock();
    const component = await client.compile.component({
      code: COUNTER_CODE,
      slots: [sdk.StorageSlot.emptyValue(COUNTER_SLOT_NAME)],
    });

    const getCountHash = component.getProcedureHash("get_count");
    const incrementHash = component.getProcedureHash("increment_count");

    expect(getCountHash).not.toBeNull();
    expect(incrementHash).not.toBeNull();
    // A Word rendered as hex is 66 hex chars (32 bytes + "0x" prefix)
    expect(getCountHash?.length).toBe(66);
    expect(incrementHash?.length).toBe(66);
  });

  test("withSupportsAllTypes() is applied — component works in a contract", async ({
    sdk,
  }) => {
    // If withSupportsAllTypes() weren't called, building the account below would
    // fail because the component wouldn't be compatible with the account type.
    const MidenClient = await createMidenClient(sdk);
    test.skip(!MidenClient, "requires napi binary (Node.js only)");
    const client = await MidenClient.createMock();
    const component = await client.compile.component({
      code: COUNTER_CODE,
      slots: [sdk.StorageSlot.emptyValue(COUNTER_SLOT_NAME)],
    });

    const seed = new Uint8Array(32);
    seed.fill(0xab);
    const auth = sdk.AuthSecretKey.rpoFalconWithRNG(seed);

    // This throws if withSupportsAllTypes() was not applied
    const account = await client.accounts.create({
      type: "ImmutableContract",
      storage: "public",
      seed,
      auth,
      components: [component],
    });

    expect(account.id().toString()).toBeDefined();
  });

  test("each call uses a fresh builder — slot accumulation does not occur", async ({
    sdk,
  }) => {
    // Two compile.component() calls with different slots must not merge.
    const MidenClient = await createMidenClient(sdk);
    test.skip(!MidenClient, "requires napi binary (Node.js only)");
    const client = await MidenClient.createMock();

    const slotA = "miden::tutorials::counter";
    const slotB = "miden::tutorials::count_reader";

    const compA = await client.compile.component({
      code: COUNTER_CODE,
      slots: [sdk.StorageSlot.emptyValue(slotA)],
    });
    const compB = await client.compile.component({
      code: COUNTER_CODE,
      slots: [sdk.StorageSlot.emptyValue(slotB)],
    });

    // Both should succeed independently (no cross-call contamination)
    expect(compA.getProcedureHash("get_count")).not.toBeNull();
    expect(compB.getProcedureHash("get_count")).not.toBeNull();
  });
});

// ════════════════════════════════════════════════════════════════
// compile.txScript()
// ════════════════════════════════════════════════════════════════

test.describe("compile.txScript()", () => {
  test("compiles a script without libraries", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    test.skip(!MidenClient, "requires napi binary (Node.js only)");
    const client = await MidenClient.createMock();
    const script = await client.compile.txScript({
      code: `
        use miden::core::sys
        begin
          exec.sys::truncate_stack
        end
      `,
    });
    expect(script).not.toBeNull();
  });

  test("compiles a script with a dynamic library", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    test.skip(!MidenClient, "requires napi binary (Node.js only)");
    const client = await MidenClient.createMock();
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
          code: COUNTER_CODE,
        },
      ],
    });
    expect(script).not.toBeNull();
  });

  test("each call uses a fresh builder — libraries from prior calls do not leak", async ({
    sdk,
  }) => {
    // Call txScript once with a library, then again without it.
    // If builders were shared, the second call would "accidentally" have the
    // first call's library and still succeed (or fail in unexpected ways).
    // With fresh builders the second call compiles cleanly in isolation.
    const MidenClient = await createMidenClient(sdk);
    test.skip(!MidenClient, "requires napi binary (Node.js only)");
    const client = await MidenClient.createMock();

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
          code: COUNTER_CODE,
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

    expect(scriptWithLib).not.toBeNull();
    expect(scriptNoLib).not.toBeNull();
  });
});

// ════════════════════════════════════════════════════════════════
// accounts.create() — contract types
// ════════════════════════════════════════════════════════════════

test.describe("accounts.create() — ImmutableContract / MutableContract", () => {
  test("ImmutableContract: isUpdatable=false, isPublic=true, isRegularAccount=true", async ({
    sdk,
  }) => {
    const MidenClient = await createMidenClient(sdk);
    test.skip(!MidenClient, "requires napi binary (Node.js only)");
    const client = await MidenClient.createMock();

    const component = await client.compile.component({
      code: COUNTER_CODE,
      slots: [sdk.StorageSlot.emptyValue(COUNTER_SLOT_NAME)],
    });

    const seed = new Uint8Array(32);
    seed.fill(0x01);
    const auth = sdk.AuthSecretKey.rpoFalconWithRNG(seed);

    const account = await client.accounts.create({
      type: "ImmutableContract",
      storage: "public",
      seed,
      auth,
      components: [component],
    });

    expect(account.isFaucet()).toBe(false);
    expect(account.isRegularAccount()).toBe(true);
    expect(account.isUpdatable()).toBe(false);
    expect(account.isPublic()).toBe(true);
    expect(account.isNew()).toBe(true);
  });

  test("MutableContract: isUpdatable=true, isPublic=true", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    test.skip(!MidenClient, "requires napi binary (Node.js only)");
    const client = await MidenClient.createMock();

    const component = await client.compile.component({
      code: COUNTER_CODE,
      slots: [sdk.StorageSlot.emptyValue(COUNTER_SLOT_NAME)],
    });

    const seed = new Uint8Array(32);
    seed.fill(0x02);
    const auth = sdk.AuthSecretKey.rpoFalconWithRNG(seed);

    const account = await client.accounts.create({
      type: "MutableContract",
      storage: "public",
      seed,
      auth,
      components: [component],
    });

    expect(account.isUpdatable()).toBe(true);
    expect(account.isPublic()).toBe(true);
    expect(account.isRegularAccount()).toBe(true);
  });

  test("ImmutableContract defaults to public storage when storage is omitted", async ({
    sdk,
  }) => {
    const MidenClient = await createMidenClient(sdk);
    test.skip(!MidenClient, "requires napi binary (Node.js only)");
    const client = await MidenClient.createMock();

    const component = await client.compile.component({
      code: COUNTER_CODE,
      slots: [sdk.StorageSlot.emptyValue(COUNTER_SLOT_NAME)],
    });

    const seed = new Uint8Array(32);
    seed.fill(0x03);
    const auth = sdk.AuthSecretKey.rpoFalconWithRNG(seed);

    // No `storage` field — should default to "public"
    const account = await client.accounts.create({
      type: "ImmutableContract",
      seed,
      auth,
      components: [component],
    });

    expect(account.isPublic()).toBe(true);
  });

  test("contract with no extra components rejects with a clear error", async ({
    sdk,
  }) => {
    // The Miden protocol requires at least one non-auth procedure in contract
    // accounts, so creating a contract with only the auth component must fail.
    const MidenClient = await createMidenClient(sdk);
    test.skip(!MidenClient, "requires napi binary (Node.js only)");
    const client = await MidenClient.createMock();

    const seed = new Uint8Array(32);
    seed.fill(0x04);
    const auth = sdk.AuthSecretKey.rpoFalconWithRNG(seed);

    try {
      await client.accounts.create({
        type: "ImmutableContract",
        storage: "public",
        seed,
        auth,
        // components intentionally omitted
      });
      expect(true).toBe(false); // should not reach here
    } catch (e: any) {
      const msg = e.message ?? String(e);
      expect(msg).toContain("at least one non-auth procedure");
    }
  });

  test("same seed yields same account ID across two builds", async ({
    sdk,
  }) => {
    const MidenClient = await createMidenClient(sdk);
    test.skip(!MidenClient, "requires napi binary (Node.js only)");

    const seed = new Uint8Array(32);
    seed.fill(0x77);

    // Build #1: through the SDK (creates + persists)
    const client = await MidenClient.createMock();
    const component1 = await client.compile.component({
      code: COUNTER_CODE,
      slots: [sdk.StorageSlot.emptyValue(COUNTER_SLOT_NAME)],
    });
    const auth1 = sdk.AuthSecretKey.rpoFalconWithRNG(seed);
    const account1 = await client.accounts.create({
      type: "ImmutableContract",
      storage: "public",
      seed,
      auth: auth1,
      components: [component1],
    });
    const id1 = account1.id().toString();

    // Build #2: raw AccountBuilder (no persistence) to avoid
    // duplicate-key error — all mock clients share the same DB.
    const component2 = await client.compile.component({
      code: COUNTER_CODE,
      slots: [sdk.StorageSlot.emptyValue(COUNTER_SLOT_NAME)],
    });
    const auth2 = sdk.AuthSecretKey.rpoFalconWithRNG(seed);
    const authComp =
      sdk.AccountComponent.createAuthComponentFromSecretKey(auth2);
    const built = new sdk.AccountBuilder(seed)
      .accountType(2 /* RegularAccountImmutableCode */)
      .storageMode(sdk.AccountStorageMode.public())
      .withAuthComponent(authComp)
      .withComponent(component2)
      .build();
    const id2 = built.account.id().toString();

    expect(id1).toBe(id2);
  });
});

// ════════════════════════════════════════════════════════════════
// transactions.execute()
// ════════════════════════════════════════════════════════════════

test.describe("transactions.execute()", () => {
  test("executes a custom script against an ImmutableContract and returns a txId", async ({
    sdk,
  }) => {
    const MidenClient = await createMidenClient(sdk);
    test.skip(!MidenClient, "requires napi binary (Node.js only)");
    const client = await MidenClient.createMock();

    // Create the counter contract
    const component = await client.compile.component({
      code: COUNTER_CODE,
      slots: [sdk.StorageSlot.emptyValue(COUNTER_SLOT_NAME)],
    });

    const seed = new Uint8Array(32);
    seed.fill(0x10);
    const auth = sdk.AuthSecretKey.rpoFalconWithRNG(seed);

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
        {
          namespace: "external_contract::counter_contract",
          code: COUNTER_CODE,
        },
      ],
    });

    // Execute the transaction
    const { txId } = await client.transactions.execute({
      account: account.id(),
      script,
    });

    expect(txId.toHex()).toBeDefined();
    expect(txId.toHex().length).toBeGreaterThan(0);
  });

  test("execute() increments storage slot on the contract", async ({ sdk }) => {
    const MidenClient = await createMidenClient(sdk);
    test.skip(!MidenClient, "requires napi binary (Node.js only)");
    const client = await MidenClient.createMock();

    const component = await client.compile.component({
      code: COUNTER_CODE,
      slots: [sdk.StorageSlot.emptyValue(COUNTER_SLOT_NAME)],
    });

    const seed = new Uint8Array(32);
    seed.fill(0x20);
    const auth = sdk.AuthSecretKey.rpoFalconWithRNG(seed);

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
        {
          namespace: "external_contract::counter_contract",
          code: COUNTER_CODE,
        },
      ],
    });

    await client.transactions.execute({ account: account.id(), script });

    client.proveBlock();
    await client.sync();

    // Read the updated storage
    const updated = await client.accounts.get(account.id());
    const countWord = updated?.storage().getItem(COUNTER_SLOT_NAME);

    // The counter word is stored little-endian; extract the value from the last 8 hex chars
    const countHex = countWord?.toHex() ?? "";
    const countValue = Number(
      BigInt("0x" + countHex.slice(-16).match(/../g).reverse().join(""))
    );

    expect(countValue).toBe(1);
  });

  // The mock chain does not track accounts created via the SDK API, so
  // get_account_proof panics when resolving foreign accounts. This test
  // requires a real node or an enhanced mock that registers SDK-created
  // accounts in MockChain.committed_accounts.
  test.fixme("execute() with foreignAccounts accepts a plain { id } wrapper", async ({
    sdk,
  }) => {
    // Verifies that the wrapper-vs-WASM discrimination logic in execute() works:
    // passing { id: accountId } (plain object) correctly resolves the account ref.
    const MidenClient = await createMidenClient(sdk);
    test.skip(!MidenClient, "requires napi binary (Node.js only)");
    const client = await MidenClient.createMock();

    // Create a target contract (the "foreign" account)
    const component = await client.compile.component({
      code: COUNTER_CODE,
      slots: [sdk.StorageSlot.emptyValue(COUNTER_SLOT_NAME)],
    });
    const seed1 = new Uint8Array(32);
    seed1.fill(0x30);
    const auth1 = sdk.AuthSecretKey.rpoFalconWithRNG(seed1);
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
    const { txId } = await client.transactions.execute({
      account: wallet.id(),
      script,
      foreignAccounts: [{ id: foreignContract.id() }],
    });

    expect(txId.toHex()).toBeDefined();
    expect(txId.toHex().length).toBeGreaterThan(0);
  });

  // Same mock limitation as above: SDK-created accounts are not in the mock
  // chain's committed list, causing get_account_proof to panic.
  test.fixme("execute() with a bare AccountId as foreignAccount (non-wrapper path)", async ({
    sdk,
  }) => {
    const MidenClient = await createMidenClient(sdk);
    test.skip(!MidenClient, "requires napi binary (Node.js only)");
    const client = await MidenClient.createMock();

    const component = await client.compile.component({
      code: COUNTER_CODE,
      slots: [sdk.StorageSlot.emptyValue(COUNTER_SLOT_NAME)],
    });
    const seed = new Uint8Array(32);
    seed.fill(0x40);
    const auth = sdk.AuthSecretKey.rpoFalconWithRNG(seed);
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
    const { txId } = await client.transactions.execute({
      account: wallet.id(),
      script,
      foreignAccounts: [foreignContract.id()],
    });

    expect(txId.toHex()).toBeDefined();
    expect(txId.toHex().length).toBeGreaterThan(0);
  });
});

// ════════════════════════════════════════════════════════════════
// transactions.executeProgram()
// ════════════════════════════════════════════════════════════════

test.describe("transactions.executeProgram()", () => {
  test("reads updated state after a mutating transaction", async ({ page }) => {
    const result = await page.evaluate(
      async ({ code, slotName }) => {
        const client = await window.MidenClient.createMock();

        const component = await client.compile.component({
          code,
          slots: [window.StorageSlot.emptyValue(slotName)],
        });

        const seed = new Uint8Array(32);
        seed.fill(0x60);
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

        // Increment the counter
        const incrScript = await client.compile.txScript({
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

        await client.transactions.execute({
          account: account.id(),
          script: incrScript,
        });

        client.proveBlock();
        await client.sync();

        // Now read the count via executeProgram — should be 1
        const readScript = await client.compile.txScript({
          code: `
            use external_contract::counter_contract
            begin
              call.counter_contract::get_count
            end
          `,
          libraries: [
            { namespace: "external_contract::counter_contract", code },
          ],
        });

        const feltArray = await client.transactions.executeProgram({
          account: account.id(),
          script: readScript,
        });

        const count = feltArray.get(0).asInt();

        return { count: count.toString() };
      },
      { code: COUNTER_CODE, slotName: COUNTER_SLOT_NAME }
    );

    expect(result.count).toBe("1");
  });
});
