// Compile-only type checks for the web-client public API surface.
//
// This file is intentionally named `.check.ts`, not `.test.ts`, so it is
// type-checked by `yarn check:api-types` but skipped by Playwright (which
// matches `*.test.ts`). The body is wrapped in a function that is never
// executed — only its types are verified.
//
// Why this file exists:
//   `js/types/api-types.d.ts` is hand-written and can drift from the actual
//   runtime behavior implemented in `js/standalone.js` and friends. When that
//   happens the package typechecks internally but breaks at real consumer
//   call sites (see #2042, where `createP2IDNote` was declared `OutputNote`
//   but actually returned `Note`). Each block below models a representative
//   consumer call site so drift fails in CI before the package ships.
//
// Adding a new standalone wrapper? Add a block here that feeds its result
// into the API a real consumer would use it with.

import {
  AccountId,
  Asset,
  Note,
  NoteArray,
  NoteTag,
  buildSwapTag,
  createP2IDENote,
  createP2IDNote,
} from "../dist/index";

declare const expectType: <T>(value: T) => void;

declare const accountId: AccountId;
declare const asset: Asset;

export function __apiTypeChecks(): never {
  throw new Error("api-types.check.ts is type-only and must never execute");

  // ── createP2IDNote returns Note, feedable into NoteArray ────────────────
  // Regression guard for https://github.com/0xMiden/miden-client/issues/2042.
  {
    const note = createP2IDNote({
      from: accountId,
      to: accountId,
      assets: asset,
      type: "public",
    });
    expectType<Note>(note);
    expectType<NoteArray>(new NoteArray([note]));
  }

  // ── createP2IDENote has the same output contract ────────────────────────
  {
    const note = createP2IDENote({
      from: accountId,
      to: accountId,
      assets: asset,
      type: "public",
      reclaimAfter: 0,
      timelockUntil: 0,
    });
    expectType<Note>(note);
    expectType<NoteArray>(new NoteArray([note]));
  }

  // ── buildSwapTag returns a usable NoteTag with the documented `.asU32()`
  {
    const tag = buildSwapTag({
      type: "public",
      offer: { token: accountId, amount: 1n },
      request: { token: accountId, amount: 1n },
    });
    expectType<NoteTag>(tag);
    expectType<number>(tag.asU32());
  }
}
