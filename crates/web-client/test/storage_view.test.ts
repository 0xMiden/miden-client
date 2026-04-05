import { expect } from "@playwright/test";
import test from "./playwright.global.setup";

// STORAGE VIEW TESTS
// =======================================================================================================
// Tests that account.storage() returns a StorageView with correct behavior
// for both Value and StorageMap slots.

test.describe("StorageView", () => {
  test("getItem() on a Value slot returns a StorageResult with correct toNumber()", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();

      const SLOT_NAME = "test::counter";
      const code = `
        use.miden::protocol::active_account
        use.miden::protocol::native_account
        use.miden::core::sys

        const COUNTER_SLOT = word("${SLOT_NAME}")

        pub proc get_count
          push.COUNTER_SLOT[0..2] exec.active_account::get_item
          exec.sys::truncate_stack
        end

        pub proc increment_count
          push.COUNTER_SLOT[0..2] exec.active_account::get_item
          add.1
          push.COUNTER_SLOT[0..2] exec.native_account::set_item
          exec.sys::truncate_stack
        end
      `;

      const component = await client.compile.component({
        code,
        slots: [window.StorageSlot.emptyValue(SLOT_NAME)],
      });

      const seed = new Uint8Array(32);
      seed.fill(0x30);
      const auth = window.AuthSecretKey.rpoFalconWithRNG(seed);

      const account = await client.accounts.create({
        type: "ImmutableContract",
        storage: "public",
        seed,
        auth,
        components: [component],
      });

      const storage = account.storage();
      const slotNames = storage.getSlotNames();
      const item = storage.getItem(SLOT_NAME);

      return {
        hasRaw: storage.raw !== undefined,
        slotNames,
        isMap: item?.isMap,
        number: item?.toNumber(),
        bigint: item?.toBigInt()?.toString(),
        hex: item?.toHex(),
        str: item?.toString(),
        json: item?.toJSON(),
        valueOf: item ? +item : undefined,
        hasEntries: item?.entries !== undefined,
        hasWord: item?.word !== undefined,
        hasFelts: item?.toFelts()?.length,
      };
    });

    expect(result.hasRaw).toBe(true);
    expect(result.slotNames).toContain("test::counter");
    expect(result.isMap).toBe(false);
    expect(result.number).toBe(0);
    expect(result.bigint).toBe("0");
    expect(result.hex).toBeDefined();
    expect(result.str).toBe("0");
    expect(result.json).toBe("0");
    expect(result.valueOf).toBe(0);
    expect(result.hasEntries).toBe(false); // undefined for Value slots
    expect(result.hasWord).toBe(true);
    expect(result.hasFelts).toBe(4);
  });

  test("getItem() on a StorageMap slot returns a StorageResult with entries", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();

      const SLOT_NAME = "test::balances";
      const component = await client.compile.component({
        code: `
          use.miden::protocol::active_account
          use.miden::core::sys

          const MAP_SLOT = word("${SLOT_NAME}")

          pub proc get_balance
            push.MAP_SLOT[0..2] exec.active_account::get_map_item
            exec.sys::truncate_stack
          end
        `,
        slots: [window.StorageSlot.map(SLOT_NAME, new window.StorageMap())],
      });

      const seed = new Uint8Array(32);
      seed.fill(0x31);
      const auth = window.AuthSecretKey.rpoFalconWithRNG(seed);

      const account = await client.accounts.create({
        type: "ImmutableContract",
        storage: "public",
        seed,
        auth,
        components: [component],
      });

      const storage = account.storage();
      const item = storage.getItem(SLOT_NAME);

      return {
        isMap: item?.isMap,
        entriesType: item?.entries !== undefined ? "array" : "undefined",
        entriesLength: item?.entries?.length,
        // Empty map — toNumber should be 0
        number: item?.toNumber(),
      };
    });

    expect(result.isMap).toBe(true);
    expect(result.entriesType).toBe("array");
    expect(result.entriesLength).toBe(0); // Empty map
    expect(result.number).toBe(0);
  });

  test("getCommitment() returns the raw commitment hash", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();

      const SLOT_NAME = "test::value";
      const component = await client.compile.component({
        code: `
          use.miden::protocol::active_account
          use.miden::core::sys

          const SLOT = word("${SLOT_NAME}")

          pub proc read
            push.SLOT[0..2] exec.active_account::get_item
            exec.sys::truncate_stack
          end
        `,
        slots: [window.StorageSlot.emptyValue(SLOT_NAME)],
      });

      const seed = new Uint8Array(32);
      seed.fill(0x32);
      const auth = window.AuthSecretKey.rpoFalconWithRNG(seed);

      const account = await client.accounts.create({
        type: "ImmutableContract",
        storage: "public",
        seed,
        auth,
        components: [component],
      });

      const storage = account.storage();
      const commitment = storage.getCommitment(SLOT_NAME);
      const rawItem = storage.raw.getItem(SLOT_NAME);

      return {
        commitmentHex: commitment?.toHex(),
        rawHex: rawItem?.toHex(),
        match: commitment?.toHex() === rawItem?.toHex(),
      };
    });

    expect(result.commitmentHex).toBeDefined();
    expect(result.rawHex).toBeDefined();
    expect(result.match).toBe(true);
  });

  test("getNumber() returns number directly", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();

      const SLOT_NAME = "test::num";
      const component = await client.compile.component({
        code: `
          use.miden::protocol::active_account
          use.miden::core::sys

          const SLOT = word("${SLOT_NAME}")

          pub proc read
            push.SLOT[0..2] exec.active_account::get_item
            exec.sys::truncate_stack
          end
        `,
        slots: [window.StorageSlot.emptyValue(SLOT_NAME)],
      });

      const seed = new Uint8Array(32);
      seed.fill(0x33);
      const auth = window.AuthSecretKey.rpoFalconWithRNG(seed);

      const account = await client.accounts.create({
        type: "ImmutableContract",
        storage: "public",
        seed,
        auth,
        components: [component],
      });

      return {
        number: account.storage().getNumber(SLOT_NAME),
        nonExistent: account.storage().getNumber("does::not::exist"),
      };
    });

    expect(result.number).toBe(0);
    expect(result.nonExistent).toBeUndefined();
  });
});
