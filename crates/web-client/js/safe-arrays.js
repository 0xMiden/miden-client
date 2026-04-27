/**
 * Non-consuming wrappers for wasm-bindgen-generated array classes.
 *
 * The default wasm-bindgen-generated constructor for an exported `Vec<T>`
 * parameter (e.g. `pub fn new(elements: Option<Vec<Note>>) -> Self`) takes
 * each input element by value: the Rust-side value is moved out of the
 * caller's JS handle. The handle is left dangling — its `__wbg_ptr` field
 * is unchanged so the JS object looks fine, but any subsequent method call
 * panics inside WASM with the opaque `"null pointer passed to rust"`
 * error from wasm-bindgen.
 *
 * That's a footgun for JS users, who don't expect "this object can no
 * longer be used" semantics from a constructor like
 * `new NoteArray([note])`. So we wrap each affected array with a class
 * that builds the same array via `push(&T)` — which already borrows +
 * clones — leaving the originals fully usable afterwards.
 *
 * The wrapper extends the wasm-bindgen base class, so `instanceof` checks
 * (including `_assertClass(...)` in other auto-generated wasm-bindgen
 * methods) keep working transparently.
 */

import {
  AccountArray as _AccountArray,
  AccountIdArray as _AccountIdArray,
  FeltArray as _FeltArray,
  ForeignAccountArray as _ForeignAccountArray,
  NoteAndArgsArray as _NoteAndArgsArray,
  NoteArray as _NoteArray,
  NoteIdAndArgsArray as _NoteIdAndArgsArray,
  NoteRecipientArray as _NoteRecipientArray,
  OutputNoteArray as _OutputNoteArray,
  StorageSlotArray as _StorageSlotArray,
  TransactionScriptInputPairArray as _TransactionScriptInputPairArray,
} from "../Cargo.toml";

function makeSafeArray(Base) {
  return class extends Base {
    constructor(elements) {
      super(); // empty Rust Vec — no consume
      if (Array.isArray(elements)) {
        for (const el of elements) {
          // push(&T) on Base borrows and clones — input handles stay valid.
          this.push(el);
        }
      }
    }
  };
}

export const AccountArray = makeSafeArray(_AccountArray);
export const AccountIdArray = makeSafeArray(_AccountIdArray);
export const FeltArray = makeSafeArray(_FeltArray);
export const ForeignAccountArray = makeSafeArray(_ForeignAccountArray);
export const NoteAndArgsArray = makeSafeArray(_NoteAndArgsArray);
export const NoteArray = makeSafeArray(_NoteArray);
export const NoteIdAndArgsArray = makeSafeArray(_NoteIdAndArgsArray);
export const NoteRecipientArray = makeSafeArray(_NoteRecipientArray);
export const OutputNoteArray = makeSafeArray(_OutputNoteArray);
export const StorageSlotArray = makeSafeArray(_StorageSlotArray);
export const TransactionScriptInputPairArray = makeSafeArray(
  _TransactionScriptInputPairArray
);
