import rust from "@wasm-tool/rollup-plugin-rust";
import resolve from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";
import copy from "rollup-plugin-copy";

// Flag that indicates if the build is meant for development purposes.
// If true, wasm-opt is not applied.
const devMode = process.env.MIDEN_WEB_DEV === "true";

// Arguments to tell cargo to add full debug symbols
// to the generated .wasm file (dev mode only).
// Note: strip='none' is already set by cargoArgsLineTablesDebug.
const cargoArgsUseDebugSymbols = ["--config", "profile.release.debug='full'"];

// Lightweight debug info for readable stack traces (always applied).
// Produces function names and line numbers with minimal size overhead.
const cargoArgsLineTablesDebug = [
  "--config",
  "profile.release.debug='line-tables-only'",
  "--config",
  "profile.release.strip='none'",
];

const wasmOptArgs = [
  // Strip DWARF sections before optimization to avoid binaryen crashes on
  // unsupported DWARF versions. The name section (function names) is kept.
  "--strip-dwarf",
  devMode ? "-O0" : "-O3",
  "--enable-bulk-memory",
  "--enable-nontrapping-float-to-int",
  // Preserve the name section through optimization passes.
  "--debuginfo",
];

// Base cargo arguments
const baseCargoArgs = [
  "--features",
  "testing",
  "--config",
  `build.rustflags=["-C", "target-feature=+atomics,+bulk-memory,+mutable-globals", "-C", "link-arg=--max-memory=4294967296", "-C", "panic=abort"]`,
  "--no-default-features",
  // Always include line-tables-only debug info for readable stack traces.
  ...cargoArgsLineTablesDebug,
  // In dev mode, append full debug symbols AFTER line-tables-only.
  // Cargo uses last-wins semantics for repeated --config keys,
  // so debug='full' overrides debug='line-tables-only'.
].concat(devMode ? cargoArgsUseDebugSymbols : []);

/**
 * Rollup configuration file for building a Cargo project and creating a WebAssembly (WASM) module,
 * as well as bundling a dedicated web worker file.
 *
 * The configuration sets up three build processes:
 *
 * 1. **WASM Module Build:**
 *    Compiles Rust code into WASM using the @wasm-tool/rollup-plugin-rust plugin. This process
 *    applies specific cargo arguments to enable necessary WebAssembly features (such as atomics,
 *    bulk memory operations, and mutable globals) and to set maximum memory limits. For testing builds,
 *    the WASM optimization level is set to 0 to improve build times, reducing the feedback loop during development.
 *
 * 2. **Worker Build:**
 *    Bundles the dedicated web worker file (`web-client-methods-worker.js`) into the `dist/workers` directory.
 *    This configuration resolves WASM module imports and uses the copy plugin to ensure that the generated
 *    WASM assets are available to the worker.
 *
 * 3. **Main Entry Point Build:**
 *    Resolves and bundles the main JavaScript file (`index.js`) for the primary entry point of the application
 *    into the `dist` directory.
 *
 * Each build configuration outputs ES module format files with source maps to facilitate easier debugging.
 */
export default [
  {
    input: ["./js/wasm.js", "./js/index.js"],
    output: {
      dir: `dist`,
      format: "es",
      sourcemap: true,
      assetFileNames: "assets/[name][extname]",
    },
    plugins: [
      rust({
        verbose: true,
        extraArgs: {
          cargo: [...baseCargoArgs],
          wasmOpt: wasmOptArgs,
          wasmBindgen: ["--keep-debug"],
        },
        experimental: {
          typescriptDeclarationDir: "dist/crates",
        },
        optimize: { release: true, rustc: !devMode },
      }),
      resolve(),
      commonjs(),
      // Convert the top-level `await __wbg_init(...)` to a non-blocking
      // exported Promise. This prevents the TLA from blocking WKWebView
      // module evaluation while still allowing the Worker (and anyone else)
      // to await WASM initialization explicitly via `wasmReady`.
      //
      // Before: `await __wbg_init({ module_or_path: url });`  (TLA — blocks)
      // After:  `var wasmReady = __wbg_init({ module_or_path: url });` (fire-and-forget)
      //         + exported as `wasmReady` for explicit awaiting
      {
        name: "remove-wasm-tla",
        generateBundle(_, bundle) {
          for (const [name, chunk] of Object.entries(bundle)) {
            if (chunk.type !== "chunk" || !chunk.code) continue;
            if (!chunk.code.includes("__wbg_init")) continue;
            const before = chunk.code.length;
            // Simply remove the TLA line
            chunk.code = chunk.code.replace(
              /\n\s*await __wbg_init\([^)]*\);\s*\n/g,
              "\n"
            );
            if (chunk.code.length !== before) {
              // Export __wbg_init and the WASM URL so loadWasm() can call
              // __wbg_init with the correct URL explicitly.
              // Only add if not already exported (prevent double-apply).
              if (
                !chunk.code.includes("__wbg_init,") &&
                !chunk.code.includes(", __wbg_init")
              ) {
                chunk.code = chunk.code.replace(
                  /export \{([^}]+)\};(\s*)$/m,
                  "export { $1, __wbg_init, module$$1 as __wasm_url };$2"
                );
              }
              console.log(`[remove-wasm-tla] Stripped TLA from ${name}`);
            }
          }
        },
      },
    ],
  },
  // Classic worker build.
  //
  // Safari/WKWebView is extremely slow with module workers ({type: "module"}),
  // so we ship a self-contained async-IIFE classic script alongside the module
  // variant below. `wrap-worker-classic` rewrites `import.meta.url` →
  // `self.location.href` (the only form a classic worker can see at runtime),
  // strips `export` clauses, and wraps the rollup ESM output in an async IIFE.
  //
  // Output: dist/workers/web-client-methods-worker.js
  {
    input: "./js/workers/web-client-methods-worker.js",
    output: {
      dir: `dist/workers`,
      format: "es",
      sourcemap: true,
      inlineDynamicImports: true,
    },
    plugins: [
      resolve(),
      commonjs(),
      copy({
        targets: [
          // Copy WASM to `dist/workers/assets` for worker accessibility
          { src: "dist/assets/*.wasm", dest: "dist/workers/assets" },
        ],
        verbose: true,
      }),
      // Wrap the worker in an async IIFE so it works as a classic script.
      // Replace ESM-only constructs (import.meta, export) with compatible alternatives.
      {
        name: "wrap-worker-classic",
        generateBundle(_, bundle) {
          for (const [, chunk] of Object.entries(bundle)) {
            if (chunk.type !== "chunk" || !chunk.code) continue;
            // Replace import.meta references for classic script compatibility.
            // Downstream bundlers (Vite) will transform these URLs before our
            // replacement runs, so the hashed paths are preserved.
            chunk.code = chunk.code.replace(
              /import\.meta\.url/g,
              "self.location.href"
            );
            chunk.code = chunk.code.replace(/import\.meta\.env/g, "undefined");
            chunk.code = chunk.code.replace(/^export\s*\{[^}]*\};?\s*$/gm, "");
            chunk.code = chunk.code.replace(
              /^export\s+default\s+/gm,
              "var _default = "
            );
            chunk.code = chunk.code.replace(
              /^export\s+(const|let|var|function|class|async)\s/gm,
              "$1 "
            );
            chunk.code = "(async function() {\n" + chunk.code + "\n})();";
          }
        },
      },
    ],
  },
  // Module worker build.
  //
  // Same input as above, but emitted as a plain ES module (.mjs) without the
  // classic IIFE wrapping. `import.meta.url` is preserved, which lets webpack
  // 5's asset tracer statically resolve `new URL("assets/miden_client_web.wasm",
  // import.meta.url)` inside the Cargo-bindgen glue and copy the WASM file into
  // the bundler's output correctly. Issue #2046: v0.14.1's classic-only worker
  // rewrites that reference to `self.location.href`, which webpack cannot trace,
  // producing a 404 on the WASM file for Next.js/webpack consumers.
  //
  // Output: dist/workers/web-client-methods-worker.mjs
  {
    input: "./js/workers/web-client-methods-worker.js",
    output: {
      dir: `dist/workers`,
      format: "es",
      sourcemap: true,
      // Two deliberate choices here:
      //
      // 1. NOT inlining dynamic imports. Keeping the
      //    `await import("./Cargo-*.js")` as a real dynamic ESM import
      //    lets webpack's module-graph analysis follow the Cargo glue and
      //    copy the sibling `miden_client_web.wasm` that the glue references
      //    via `new URL("assets/...", import.meta.url)`. With
      //    `inlineDynamicImports`, the URL literal ends up buried inside the
      //    worker bundle and webpack's worker sub-compilation never sees it
      //    as a graph dependency.
      //
      // 2. `.js` extension, not `.mjs`. Webpack 5 routes `.mjs` worker files
      //    through `type: "asset/resource"` (copy-only, no sub-compilation),
      //    so dynamic imports inside them never get chunked and runtime fetch
      //    404s on the Cargo glue. `.js` with `{ type: "module" }` on the
      //    Worker constructor triggers the proper worker sub-compilation and
      //    all chunking works. The `.module` infix disambiguates this file
      //    from the classic worker output that sits alongside it.
      entryFileNames: "[name].module.js",
    },
    plugins: [resolve(), commonjs()],
  },
];
