# Web Client Error Pipeline — Analysis & Fixes

## The Error Pipeline (and where it breaks)

```
Rust enum variant (rich, typed, chained)
  ↓ js_error_with_context() flattens chain to a single string
JsError with .message + .help property
  ↓ postMessage to worker
Serialized {name, message, stack, cause}  ← .help is LOST here
  ↓ deserializeError()
JS Error object the consumer sees         ← no hint, just a flat string
```

---

## Boundary 1: WASM — enum variants become flat strings

`js_error_with_context()` in `crates/web-client/src/lib.rs` walks the full error chain and concatenates everything with `": "`. The result is a single string like:

> "failed to execute transaction: transaction execution failed: assertion failed: ..."

The Rust enum variant name (e.g. `InsufficientFunds`, `AccountLocked`) is gone. All you get is whatever the `Display` impl produces. This is inherent to `wasm-bindgen` — `JsError` only takes a message string.

## Boundary 2: Worker postMessage — hints are silently dropped

This is the biggest bug. The hint system works — `Reflect::set()` attaches a `.help` property to the JsError in Rust. But the worker serializer only copies standard Error fields:

```js
// workers/web-client-methods-worker.js
return {
  name: error.name,
  message: error.message,
  stack: error.stack,
  cause: error.cause ? serializeError(error.cause) : undefined,
  code: error.code,
};
// ← .help is never copied
```

**Every error that goes through the worker (all heavy ops — execute, prove, submit, sync) loses its hint.** Only errors from lightweight main-thread operations retain hints.

## Boundary 3: Panics produce garbage

There are `panic!()` calls and `.unwrap()` sites in the web-client models that bypass the entire error pipeline:

- `crates/web-client/src/models/note_filter.rs` — 4 `panic!()` calls
- `crates/web-client/src/models/transaction_request/transaction_request_builder.rs` — `.build().unwrap()`

These produce `"RuntimeError: unreachable"` or a raw stack trace with no context whatsoever.

---

## Why only *some* errors are cryptic

| Error path | Quality | Why |
|---|---|---|
| Main-thread error with a hint | Good | Full chain + hint survives |
| Main-thread error without a hint | Okay | Chain is readable but no actionable guidance |
| Worker error with a hint | Bad | **Hint silently dropped** by serializer |
| Worker error without a hint | Bad | Just a flattened string with no guidance |
| Upstream crate error (miden-tx) | Bad | Display impls are terse, no hints defined |
| Panic/unwrap | Terrible | `"unreachable"` or raw stack trace |

---

## The hint coverage gap

Only **~8 out of 30+ `ClientError` variants** have hints. And `hint_from_error()` uses `downcast_ref::<ClientError>()` — so if the error is wrapped by an upstream type, the downcast fails and returns `None` even for errors that *do* have hints.

---

## Fixes (ordered by impact)

### 1. Add `help` to the worker serializer (highest impact, one-line fix)

Instantly recovers all existing hints for worker-path errors (execute, prove, submit, sync).

### 2. Replace panics with proper `Result` returns

Convert `panic!()` and `.unwrap()` in model files to return `Result<T, JsValue>` errors with context.

### 3. Expand hint coverage

Add hints to more error variants, especially:
- `InsufficientFunds`
- `InvalidInputNotes`
- `InvalidAccountState`
- All `TransactionProverError` variants
- All `NoteCheckerError` variants

### 4. Use `{err}` (Display) not `{err:?}` (Debug)

A few places (e.g. `export.rs`) use Debug format which produces raw Rust Debug output instead of user-friendly messages.

---

## Key files in the error pipeline

1. **Rust error definitions:** `crates/rust-client/src/errors.rs`
2. **Error-to-JsValue conversion:** `crates/web-client/src/lib.rs` (`js_error_with_context`, `hint_from_error`)
3. **WASM method error handling:** `crates/web-client/src/new_transactions.rs` (and all other WASM modules)
4. **Worker error serialization:** `crates/web-client/js/workers/web-client-methods-worker.js`
5. **Worker error deserialization:** `crates/web-client/js/index.js` (`deserializeError`)
6. **Panics in web-client:** `crates/web-client/src/models/note_filter.rs`, `crates/web-client/src/models/transaction_request/transaction_request_builder.rs`
