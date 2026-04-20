// Eager entry point for @miden-sdk/miden-sdk.
//
// Awaits WASM initialization at module top level, so importing this module
// guarantees that any wasm-bindgen constructor (`new RpcClient(...)`,
// `AccountId.fromHex(...)`, `TransactionProver.newRemoteProver(...)`, etc.)
// is safe to call synchronously on the next line. No explicit
// `await MidenClient.ready()` / `isReady` gate is required.
//
// This is the default entry (`@miden-sdk/miden-sdk` → `./dist/eager.js`).
//
// When NOT to use this entry:
// - **Capacitor mobile apps** (Miden Wallet iOS/Android): Capacitor's
//   `capacitor://localhost` scheme handler interacts poorly with top-level
//   await in the main WKWebView. Verified empirically: TLA in a Capacitor
//   host WKWebView hangs module evaluation indefinitely, while the same
//   TLA in the dApp-browser WKWebView (vanilla HTTPS) resolves in <100ms.
// - **Next.js / SSR**: TLA blocks server-side module evaluation.
// - **Framework adapters (@miden-sdk/react, etc.)**: they manage readiness
//   via their own state machine (e.g. `isReady`) and should not impose
//   TLA on consumer bundles.
//
// For those contexts, import from `@miden-sdk/miden-sdk/lazy` — identical
// API surface, no top-level await, callers are responsible for awaiting
// `MidenClient.ready()` (or the equivalent) before touching wasm-bindgen
// types.
import { getWasmOrThrow } from "./index.js";

await getWasmOrThrow();

export * from "./index.js";
