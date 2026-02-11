// Node.js entry point for the Miden web-client SDK.
// Polyfills (HTTP/2 fetch, file:// interception, globalThis.self) are
// injected by Rollup via output.intro in rollup.config.node.js so they
// execute before any bundled code (including WASM initialization).

// Re-export everything from the main entry
export * from "./index.js";
