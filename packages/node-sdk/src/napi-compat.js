/**
 * NAPI compatibility layer.
 *
 * Normalizes differences between the napi (Node.js) and wasm-bindgen (browser)
 * SDK surfaces so the shared MidenClient wrapper works on both platforms.
 *
 * Key normalizations:
 * - BigInt -> Number (napi uses f64 for u64, browser uses BigInt)
 * - Uint8Array/Buffer -> Array (napi's Vec<u8> expects plain arrays)
 * - null -> undefined (napi returns null for Option::None, wasm-bindgen returns undefined)
 * - camelCase -> snake_case aliases (napi uses camelCase, wasm-bindgen uses snake_case)
 * - Array type polyfills (browser has typed WASM arrays, napi accepts plain JS arrays)
 */

// ── Argument normalization ───────────────────────────────────────────

/**
 * Normalizes a single argument for napi compatibility.
 */
export function normalizeArg(val) {
  if (typeof val === "bigint") return Number(val);
  if (val instanceof BigUint64Array) return Array.from(val, (v) => Number(v));
  if (val instanceof BigInt64Array) return Array.from(val, (v) => Number(v));
  if (val instanceof Uint8Array || Buffer.isBuffer(val)) return Array.from(val);
  return val;
}

// ── Class wrapping ───────────────────────────────────────────────────

/**
 * Wraps a napi class so constructor and static method args are normalized.
 */
export function wrapClass(Cls) {
  if (!Cls) return Cls;
  const Wrapper = function (...args) {
    return new Cls(...args.map(normalizeArg));
  };
  Wrapper.prototype = Cls.prototype;
  for (const key of Object.getOwnPropertyNames(Cls)) {
    if (key === "prototype" || key === "length" || key === "name") continue;
    const desc = Object.getOwnPropertyDescriptor(Cls, key);
    if (desc && typeof desc.value === "function") {
      Wrapper[key] = (...args) => desc.value.apply(Cls, args.map(normalizeArg));
    } else if (desc) {
      try {
        Object.defineProperty(Wrapper, key, desc);
      } catch {
        /* skip non-configurable */
      }
    }
  }
  return Wrapper;
}

// ── Client wrapping ──────────────────────────────────────────────────

/**
 * Wraps a raw napi WebClient to normalize API differences with the browser SDK.
 *
 * - syncState() -> syncStateImpl() (no browser lock coordination needed)
 * - syncStateWithTimeout() -> syncStateImpl() (timeout not applicable)
 * - null -> undefined for Option<T> returns
 * - BigInt/Uint8Array args normalized
 */
export function wrapClient(rawClient, storeName) {
  return new Proxy(rawClient, {
    get(target, prop) {
      if (prop === "syncState") {
        return (...args) => target.syncStateImpl(...args);
      }
      if (prop === "syncStateWithTimeout") {
        return (_timeoutMs) => target.syncStateImpl();
      }
      if (prop === "storeName") {
        return storeName || "default";
      }
      if (prop === "wasmWebClient") {
        return target;
      }
      if (prop === "storeIdentifier") {
        return () => target.storeIdentifier?.() ?? storeName ?? "unknown";
      }
      // terminate is a no-op on Node.js (no Web Worker to terminate)
      if (prop === "terminate") {
        return () => {};
      }
      // onStateChanged is browser-only (uses BroadcastChannel)
      if (prop === "onStateChanged") {
        return () => undefined;
      }
      if (prop === "newWallet") {
        return (mode, mutable, authScheme, seed) => {
          const normSeed =
            seed instanceof Uint8Array || Buffer.isBuffer(seed)
              ? Array.from(seed)
              : seed;
          return target
            .newWallet(mode, mutable, authScheme, normSeed ?? null)
            .then((v) => (v === null ? undefined : v));
        };
      }
      const val = target[prop];
      if (typeof val === "function") {
        const bound = val.bind(target);
        return (...args) => {
          const result = bound(...args.map(normalizeArg));
          if (result && typeof result.then === "function") {
            return result.then((v) => (v === null ? undefined : v));
          }
          return result === null ? undefined : result;
        };
      }
      return val;
    },
  });
}

// ── Prototype patching ───────────────────────────────────────────────

/**
 * Patches the raw SDK module:
 * - Adds snake_case aliases for camelCase methods
 * - Converts null -> undefined for Option<T> returns
 * - Aliases static methods
 */
export function patchSdkPrototypes(rawSdk) {
  // snake_case aliases for instance methods
  for (const [cls, aliases] of [
    [rawSdk.Account, { to_commitment: "toCommitment" }],
    [rawSdk.AccountHeader, { to_commitment: "toCommitment" }],
  ]) {
    if (!cls?.prototype) continue;
    for (const [snake, camel] of Object.entries(aliases)) {
      if (typeof cls.prototype[camel] === "function" && !cls.prototype[snake]) {
        cls.prototype[snake] = cls.prototype[camel];
      }
    }
  }

  // null -> undefined for Option<T> return methods
  for (const [cls, methods] of [
    [rawSdk.AccountStorage, ["getItem", "getMapEntries", "getMapItem"]],
    [rawSdk.NoteConsumability, ["consumableAfterBlock"]],
  ]) {
    if (!cls?.prototype) continue;
    for (const method of methods) {
      const original = cls.prototype[method];
      if (typeof original === "function") {
        cls.prototype[method] = function (...args) {
          const result = original.apply(this, args);
          return result === null ? undefined : result;
        };
      }
    }
  }

  // snake_case aliases for static methods
  if (rawSdk.NoteScript) {
    if (!rawSdk.NoteScript.p2id && rawSdk.NoteScript.p2Id)
      rawSdk.NoteScript.p2id = rawSdk.NoteScript.p2Id;
    if (!rawSdk.NoteScript.p2ide && rawSdk.NoteScript.p2Ide)
      rawSdk.NoteScript.p2ide = rawSdk.NoteScript.p2Ide;
  }
}

// ── Array polyfills ──────────────────────────────────────────────────

/**
 * Creates polyfill constructors for WASM typed array types.
 * napi accepts plain JS arrays directly, but the browser SDK requires
 * typed wrappers (NoteAndArgsArray, FeltArray, etc.). These polyfills
 * let `new sdk.FeltArray([a, b])` work on Node.js by returning a plain array.
 */
export function makeArrayPolyfills() {
  function polyfill(items) {
    const arr = Array.isArray(items) ? [...items] : [items];
    arr.get = (i) => arr[i];
    arr.replaceAt = (i, val) => {
      arr[i] = val;
      return arr;
    };
    return arr;
  }
  const names = [
    "AccountArray",
    "AccountIdArray",
    "FeltArray",
    "ForeignAccountArray",
    "NoteAndArgsArray",
    "NoteDetailsAndTagArray",
    "NoteIdAndArgsArray",
    "NoteRecipientArray",
    "OutputNoteArray",
    "OutputNotesArray",
    "StorageSlotArray",
    "TransactionScriptInputPairArray",
  ];
  const result = {};
  for (const name of names) {
    result[name] = polyfill;
  }
  return result;
}

// ── SDK wrapper ──────────────────────────────────────────────────────

/**
 * Creates a wrapped SDK module suitable for use with the MidenClient wrapper.
 * Applies all patches and returns an object that can be used as `getWasm()` return value.
 */
export function createSdkWrapper(rawSdk) {
  patchSdkPrototypes(rawSdk);

  return {
    ...rawSdk,
    // Wrap classes whose constructors/static methods accept BigInt or Uint8Array
    AccountBuilder: wrapClass(rawSdk.AccountBuilder),
    AccountComponent: wrapClass(rawSdk.AccountComponent),
    AuthSecretKey: wrapClass(rawSdk.AuthSecretKey),
    Felt: wrapClass(rawSdk.Felt),
    FungibleAsset: wrapClass(rawSdk.FungibleAsset),
    Word: wrapClass(rawSdk.Word),
    NoteTag: wrapClass(rawSdk.NoteTag),
    // Array type polyfills
    ...makeArrayPolyfills(),
  };
}
