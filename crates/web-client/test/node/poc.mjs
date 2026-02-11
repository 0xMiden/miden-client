/**
 * PoC: Load the Miden web-client WASM in Node.js
 *
 * Validates:
 * 1. WASM loads and initializes in Node.js
 * 2. Dexie/IndexedDB storage works via fake-indexeddb polyfill
 * 3. WASM types can be constructed and used (crypto, accounts, etc.)
 * 4. gRPC-Web RPC round-trip works (tonic-web-wasm-client via fetch)
 * 5. crypto.getRandomValues works (getrandom wasm_js backend)
 *
 * Usage: node poc.mjs [rpc-url]
 *   Default RPC: https://rpc.devnet.miden.io
 */

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { resolve, dirname } from "node:path";
import { Agent, setGlobalDispatcher } from "undici";

const RPC_URL = process.argv[2] || "https://rpc.devnet.miden.io";

let passed = 0;
let failed = 0;

function pass(name, detail) {
  passed++;
  const msg = detail ? `[PASS] ${name}: ${detail}` : `[PASS] ${name}`;
  console.log(msg);
}

function fail(name, detail) {
  failed++;
  const msg = detail ? `[FAIL] ${name}: ${detail}` : `[FAIL] ${name}`;
  console.error(msg);
}

function info(msg) {
  console.log(`[info] ${msg}`);
}

// ─── Step 1: Polyfill IndexedDB ─────────────────────────────────────────────
import "fake-indexeddb/auto";
pass("IndexedDB polyfill", "fake-indexeddb loaded");

// ─── Step 2: Enable HTTP/2 for fetch ────────────────────────────────────────
// Node.js built-in fetch (undici) defaults to HTTP/1.1, but the Miden gRPC-Web
// server requires HTTP/2 (browsers negotiate this via ALPN automatically).
setGlobalDispatcher(new Agent({ allowH2: true }));
pass("HTTP/2 fetch", "undici Agent with allowH2 enabled");

// ─── Step 3: Patch fetch for file:// URLs ───────────────────────────────────
// wasm-bindgen uses: fetch(new URL("assets/foo.wasm", import.meta.url))
// Node.js fetch doesn't support file:// URLs, so we intercept them.
const _originalFetch = globalThis.fetch;
const fetchLog = [];

globalThis.fetch = async function patchedFetch(input, init) {
  let url;
  if (input instanceof URL) {
    url = input;
  } else if (input instanceof Request) {
    url = new URL(input.url);
  } else if (typeof input === "string") {
    try {
      url = new URL(input);
    } catch {
      return _originalFetch(input, init);
    }
  }

  if (url && url.protocol === "file:") {
    const filePath = fileURLToPath(url);
    info(`Intercepted file:// fetch -> ${filePath}`);
    const buffer = readFileSync(filePath);
    return new Response(buffer, {
      status: 200,
      headers: { "Content-Type": "application/wasm" },
    });
  }

  const reqUrl = url?.href || (typeof input === "string" ? input : input?.url);
  const reqHeaders = {};
  if (input instanceof Request) {
    input.headers.forEach((v, k) => { reqHeaders[k] = v; });
  } else if (init?.headers) {
    if (init.headers instanceof Headers) {
      init.headers.forEach((v, k) => { reqHeaders[k] = v; });
    } else {
      Object.assign(reqHeaders, init.headers);
    }
  }

  const response = await _originalFetch(input, init);

  const respHeaders = {};
  response.headers.forEach((v, k) => { respHeaders[k] = v; });
  fetchLog.push({ url: reqUrl, reqHeaders, status: response.status, respHeaders });

  return response;
};
pass("fetch patched", "file:// URL interception active");

// ─── Step 4: Polyfill missing browser globals ───────────────────────────────
if (typeof globalThis.self === "undefined") {
  globalThis.self = globalThis;
  pass("globalThis.self", "polyfilled");
} else {
  pass("globalThis.self", "already defined");
}

// ─── Step 5: Import the built web-client ────────────────────────────────────
info("Importing dist/index.js (loads & instantiates WASM)...");

const distDir = resolve(
  dirname(fileURLToPath(import.meta.url)),
  "../../dist"
);

let sdk;
try {
  sdk = await import(`${distDir}/index.js`);
  pass("WASM load", `module loaded (${Object.keys(sdk).length} exports)`);
} catch (err) {
  fail("WASM load", err.message);
  printSummary();
  process.exit(1);
}

// ─── Step 6: Test WASM type construction (no server needed) ─────────────────
try {
  const endpoint = new sdk.Endpoint(RPC_URL);
  pass("Endpoint construction", `${endpoint.toString()}`);
} catch (err) {
  fail("Endpoint construction", err.message);
}

try {
  const felt = new sdk.Felt(42n);
  pass("Felt construction", "Felt(42) created");
} catch (err) {
  fail("Felt construction", err.message);
}

try {
  const hash = sdk.Rpo256.hashElements(new sdk.FeltArray([
    new sdk.Felt(1n), new sdk.Felt(2n), new sdk.Felt(3n), new sdk.Felt(4n)
  ]));
  pass("Crypto (Rpo256)", "hash computed successfully");
} catch (err) {
  fail("Crypto (Rpo256)", err.message);
}

try {
  const networkId = sdk.NetworkId.devnet();
  pass("NetworkId", "devnet created");
} catch (err) {
  fail("NetworkId", err.message);
}

// ─── Step 7: Create client and test RPC (WebClient API on main branch) ──────
info(`Creating WebClient with RPC: ${RPC_URL}...`);

try {
  const client = await sdk.WebClient.createClient(RPC_URL);
  pass("Client creation", "WebClient created successfully");

  // getSyncHeight reads from local store (no RPC needed after createClient)
  try {
    const height = await client.getSyncHeight();
    pass("getSyncHeight()", `returned ${height}`);
  } catch (err) {
    fail("getSyncHeight()", err.message);
  }

  // syncState() makes gRPC-Web requests
  try {
    info("Calling syncState() (gRPC-Web round-trip to devnet)...");
    const syncSummary = await client.syncState();
    pass("syncState()", "completed successfully");
  } catch (err) {
    fail("syncState()", err.message);
  }

  client.terminate();
  pass("Client terminate", "cleaned up");
} catch (err) {
  fail("Client creation", err.message);
  if (fetchLog.length > 0) {
    const lastReq = fetchLog[fetchLog.length - 1];
    info(`Last RPC: ${lastReq.status} ${lastReq.url}`);
    info(`  req headers: ${JSON.stringify(lastReq.reqHeaders)}`);
    info(`  resp headers: ${JSON.stringify(lastReq.respHeaders)}`);
  }
}

// ─── Summary ────────────────────────────────────────────────────────────────
printSummary();

function printSummary() {
  console.log("\n" + "=".repeat(60));
  console.log("PoC SUMMARY");
  console.log("=".repeat(60));
  console.log(`Node.js: ${process.version}`);
  console.log(`RPC URL: ${RPC_URL}`);
  console.log(`Passed:  ${passed}`);
  console.log(`Failed:  ${failed}`);

  if (fetchLog.length > 0) {
    console.log("\nRPC requests:");
    for (const req of fetchLog) {
      console.log(`  ${req.status} ${req.url}`);
    }
  }

  console.log("=".repeat(60));

  if (failed > 0) {
    process.exit(1);
  }
}
