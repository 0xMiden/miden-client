// @ts-nocheck
import { test, expect } from "./test-setup";

// STORAGE VIEW TESTS
// =======================================================================================================

test.describe("StorageView", () => {
  test("getItem() on a Value slot returns a StorageResult", async ({ run }) => {
    const result = await run(async ({ sdk, helpers }) => {
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

      const client = await helpers.createMidenMockClient();
      const component = await client.compile.component({
        code,
        slots: [sdk.StorageSlot.emptyValue(SLOT_NAME)],
      });

      const seed = new Uint8Array(32);
      seed.fill(0x30);
      const auth = sdk.AuthSecretKey.rpoFalconWithRNG(seed);

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
        hasRaw: storage.raw !== undefined,
        slotNames: storage.getSlotNames(),
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
    run,
  }) => {
    const result = await run(async ({ sdk, helpers }) => {
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

      const client = await helpers.createMidenMockClient();
      const component = await client.compile.component({
        code,
        slots: [sdk.StorageSlot.map(SLOT_NAME, new sdk.StorageMap())],
      });

      const seed = new Uint8Array(32);
      seed.fill(0x31);
      const auth = sdk.AuthSecretKey.rpoFalconWithRNG(seed);

      const account = await client.accounts.create({
        type: "ImmutableContract",
        storage: "public",
        seed,
        auth,
        components: [component],
      });

      const item = account.storage().getItem(SLOT_NAME);

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

  test("getCommitment() returns the raw commitment hash", async ({ run }) => {
    const result = await run(async ({ sdk, helpers }) => {
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

      const client = await helpers.createMidenMockClient();
      const component = await client.compile.component({
        code,
        slots: [sdk.StorageSlot.emptyValue(SLOT_NAME)],
      });

      const seed = new Uint8Array(32);
      seed.fill(0x32);
      const auth = sdk.AuthSecretKey.rpoFalconWithRNG(seed);

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
      };
    });

    expect(result.commitmentHex).toBeDefined();
    expect(result.rawHex).toBeDefined();
    expect(result.commitmentHex).toBe(result.rawHex);
  });

  test("wordToBigInt() round-trips known felt values losslessly", async ({
    run,
  }) => {
    // Node.js napi maps u64 to f64; constructing via `new Word([...])` would
    // clamp/panic above 2^53. `Word.fromHex` lets us build full u64 felts on
    // both platforms so this precision test runs on either backend.
    const result = await run(async ({ sdk }) => {
      function wordWithFirstFelt(v) {
        const bytes = [];
        for (let i = 0; i < 8; i++) {
          bytes.push(
            Number((v >> BigInt(i * 8)) & 0xffn)
              .toString(16)
              .padStart(2, "0")
          );
        }
        const firstFeltLe = bytes.join("");
        const zeroFeltLe = "0000000000000000";
        return sdk.Word.fromHex(
          "0x" + firstFeltLe + zeroFeltLe + zeroFeltLe + zeroFeltLe
        );
      }

      const cases = [
        0n,
        1n,
        42n,
        BigInt(Number.MAX_SAFE_INTEGER), // 2^53 - 1
        BigInt(Number.MAX_SAFE_INTEGER) + 1n, // 2^53
        (1n << 62n) - 1n, // large but below the felt modulus
      ];

      const out = [];
      for (const v of cases) {
        const word = wordWithFirstFelt(v);
        const got = sdk.wordToBigInt(word);
        out.push({ input: v.toString(), got: got.toString(), ok: got === v });
      }
      return out;
    });

    for (const c of result) {
      expect(c.ok, `wordToBigInt(${c.input}) returned ${c.got}`).toBe(true);
    }
  });

  test("valueOf() throws RangeError for values exceeding MAX_SAFE_INTEGER", async ({
    run,
  }) => {
    const result = await run(async ({ sdk }) => {
      function wordWithFirstFelt(v) {
        const bytes = [];
        for (let i = 0; i < 8; i++) {
          bytes.push(
            Number((v >> BigInt(i * 8)) & 0xffn)
              .toString(16)
              .padStart(2, "0")
          );
        }
        const firstFeltLe = bytes.join("");
        const zeroFeltLe = "0000000000000000";
        return sdk.Word.fromHex(
          "0x" + firstFeltLe + zeroFeltLe + zeroFeltLe + zeroFeltLe
        );
      }

      // Build a StorageResult around a Word whose first felt is > MAX_SAFE_INTEGER.
      // We bypass StorageView since constructing a real account with a giant
      // value would be considerably more code; the wrapper class is the unit
      // under test here.
      const big = BigInt(Number.MAX_SAFE_INTEGER) + 1n; // 2^53
      const word = wordWithFirstFelt(big);
      const result = new sdk.StorageResult(word, false, undefined, sdk.Word);

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

  test("valueOf() returns a JS number for small values", async ({ run }) => {
    const result = await run(async ({ sdk }) => {
      function wordWithFirstFelt(v) {
        const bytes = [];
        for (let i = 0; i < 8; i++) {
          bytes.push(
            Number((v >> BigInt(i * 8)) & 0xffn)
              .toString(16)
              .padStart(2, "0")
          );
        }
        const firstFeltLe = bytes.join("");
        const zeroFeltLe = "0000000000000000";
        return sdk.Word.fromHex(
          "0x" + firstFeltLe + zeroFeltLe + zeroFeltLe + zeroFeltLe
        );
      }

      const word = wordWithFirstFelt(42n);
      const result = new sdk.StorageResult(word, false, undefined, sdk.Word);
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
