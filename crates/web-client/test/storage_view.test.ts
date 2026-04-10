import { expect } from "@playwright/test";
import test from "./playwright.global.setup";

// Shared MASM code that works with the MockChain assembler
const VALUE_SLOT_CODE = (slotName: string) => `
  use miden::protocol::active_account
  use miden::protocol::native_account
  use miden::core::word
  use miden::core::sys

  const SLOT = word("${slotName}")

  pub proc get_value
    push.SLOT[0..2] exec.active_account::get_item
    exec.sys::truncate_stack
  end

  pub proc set_value
    push.SLOT[0..2] exec.native_account::set_item
    exec.sys::truncate_stack
  end
`;

const MAP_SLOT_CODE = (slotName: string) => `
  use miden::protocol::active_account
  use miden::core::word
  use miden::core::sys

  const MAP_SLOT = word("${slotName}")

  pub proc get_map_value
    push.MAP_SLOT[0..2] exec.active_account::get_map_item
    exec.sys::truncate_stack
  end
`;

// STORAGE VIEW TESTS
// =======================================================================================================

test.describe("StorageView", () => {
  test("getItem() on a Value slot returns a StorageResult", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();

      const SLOT_NAME = "test::counter";
      const code = `
        use miden::protocol::active_account
        use miden::protocol::native_account
        use miden::core::word
        use miden::core::sys

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
    expect(result.bigint).toBe("0");
    expect(result.hex).toBeDefined();
    expect(result.str).toBe("0");
    expect(result.json).toBe("0");
    expect(result.valueOf).toBe(0);
    expect(result.hasEntries).toBe(false);
    expect(result.hasWord).toBe(true);
    expect(result.hasFelts).toBe(4);
  });

  test("getItem() on a StorageMap slot returns a StorageResult with entries", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();

      const SLOT_NAME = "test::balances";
      const code = `
        use miden::protocol::active_account
        use miden::core::word
        use miden::core::sys

        const MAP_SLOT = word("${SLOT_NAME}")

        pub proc get_balance
          push.MAP_SLOT[0..2] exec.active_account::get_map_item
          exec.sys::truncate_stack
        end
      `;

      const component = await client.compile.component({
        code,
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
        bigint: item?.toBigInt()?.toString(),
      };
    });

    expect(result.isMap).toBe(true);
    expect(result.entriesType).toBe("array");
    expect(result.entriesLength).toBe(0);
    expect(result.bigint).toBe("0");
  });

  test("getCommitment() returns the raw commitment hash", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const client = await window.MidenClient.createMock();

      const SLOT_NAME = "test::value";
      const code = `
        use miden::protocol::active_account
        use miden::core::word
        use miden::core::sys

        const SLOT = word("${SLOT_NAME}")

        pub proc read
          push.SLOT[0..2] exec.active_account::get_item
          exec.sys::truncate_stack
        end
      `;

      const component = await client.compile.component({
        code,
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

  test("wordToBigInt() round-trips known felt values losslessly", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      // Construct Words with known first-felt values via the
      // BigUint64Array constructor and assert wordToBigInt round-trips them.
      const cases: bigint[] = [
        0n,
        1n,
        42n,
        BigInt(Number.MAX_SAFE_INTEGER), // 2^53 - 1, last value that fits in JS number
        BigInt(Number.MAX_SAFE_INTEGER) + 1n, // 2^53, first value that loses precision in number
        (1n << 62n) - 1n, // large but safely below the felt modulus
      ];

      const out: { input: string; got: string; ok: boolean }[] = [];
      for (const v of cases) {
        const word = new window.Word(new BigUint64Array([v, 0n, 0n, 0n]));
        const got = window.wordToBigInt(word);
        out.push({
          input: v.toString(),
          got: got.toString(),
          ok: got === v,
        });
      }
      return out;
    });

    for (const c of result) {
      expect(c.ok, `wordToBigInt(${c.input}) returned ${c.got}`).toBe(true);
    }
  });

  test("valueOf() throws RangeError for values exceeding MAX_SAFE_INTEGER", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      // Build a StorageResult around a Word whose first felt is > MAX_SAFE_INTEGER.
      // We bypass StorageView since constructing a real account with a giant
      // value would be considerably more code; the wrapper class is the unit
      // under test here.
      const big = BigInt(Number.MAX_SAFE_INTEGER) + 1n; // 2^53
      const word = new window.Word(new BigUint64Array([big, 0n, 0n, 0n]));
      const result = new window.StorageResult(
        word,
        false,
        undefined,
        window.Word
      );

      // toBigInt() and toString() must remain lossless and never throw
      const bigStr = result.toBigInt().toString();
      const stringified = result.toString();
      const json = result.toJSON();

      // valueOf() (and `+result`) must throw a RangeError
      let threw = false;
      let message = "";
      let isRangeError = false;
      try {
        void +result;
      } catch (err) {
        threw = true;
        isRangeError = err instanceof RangeError;
        message = err instanceof Error ? err.message : String(err);
      }

      return { bigStr, stringified, json, threw, isRangeError, message };
    });

    const expected = (BigInt(Number.MAX_SAFE_INTEGER) + 1n).toString();
    expect(result.bigStr).toBe(expected);
    expect(result.stringified).toBe(expected);
    expect(result.json).toBe(expected);
    expect(result.threw).toBe(true);
    expect(result.isRangeError).toBe(true);
    expect(result.message).toContain("toBigInt");
  });

  test("valueOf() returns a JS number for small values", async ({ page }) => {
    const result = await page.evaluate(async () => {
      const word = new window.Word(new BigUint64Array([42n, 0n, 0n, 0n]));
      const result = new window.StorageResult(
        word,
        false,
        undefined,
        window.Word
      );
      return {
        valueOf: +result,
        arithmetic: result * 2,
        templated: `value: ${result}`,
      };
    });

    expect(result.valueOf).toBe(42);
    expect(result.arithmetic).toBe(84);
    expect(result.templated).toBe("value: 42");
  });
});
