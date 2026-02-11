/**
 * Node.js environment setup for the Miden web-client WASM.
 *
 * Must be imported BEFORE any dist/ imports. Sets up:
 * 1. IndexedDB polyfill (fake-indexeddb)
 * 2. HTTP/2 fetch (undici allowH2 — browsers do this via ALPN)
 * 3. file:// fetch interception (for WASM loading from disk)
 * 4. globalThis.self polyfill
 */

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { Agent, setGlobalDispatcher } from "undici";

// 1. IndexedDB polyfill — Dexie.js (used by idxdb-store) requires it
import "fake-indexeddb/auto";

// 2. HTTP/2 — gRPC-Web server requires HTTP/2, Node.js fetch defaults to HTTP/1.1
setGlobalDispatcher(new Agent({ allowH2: true }));

// 3. file:// fetch — wasm-bindgen loads .wasm via fetch(new URL("...wasm", import.meta.url))
const _originalFetch = globalThis.fetch;
globalThis.fetch = async function patchedFetch(input, init) {
  let url;
  if (input instanceof URL) {
    url = input;
  } else if (input instanceof Request) {
    url = new URL(input.url);
  } else if (typeof input === "string") {
    try { url = new URL(input); } catch { return _originalFetch(input, init); }
  }

  if (url && url.protocol === "file:") {
    const buffer = readFileSync(fileURLToPath(url));
    return new Response(buffer, {
      status: 200,
      headers: { "Content-Type": "application/wasm" },
    });
  }

  return _originalFetch(input, init);
};

// 4. globalThis.self — used by Dexie and some wasm-bindgen glue code
if (typeof globalThis.self === "undefined") {
  globalThis.self = globalThis;
}
