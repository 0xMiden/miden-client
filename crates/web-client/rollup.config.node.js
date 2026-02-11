import rust from "@wasm-tool/rollup-plugin-rust";
import resolve from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";

const devMode = process.env.MIDEN_WEB_DEV === "true";

const cargoArgsUseDebugSymbols = [
  "--config",
  "profile.release.debug='full'",
  "--config",
  "profile.release.strip='none'",
];

const baseCargoArgs = [
  "--features",
  "testing",
  "--config",
  `build.rustflags=["-C", "target-feature=+atomics,+bulk-memory,+mutable-globals", "-C", "link-arg=--max-memory=4294967296", "-C", "panic=abort"]`,
  "--no-default-features",
].concat(devMode ? cargoArgsUseDebugSymbols : []);

const wasmOptArgs = [
  devMode ? "-O0" : "-O3",
  "--enable-bulk-memory",
  "--enable-nontrapping-float-to-int",
];

// Polyfills injected at the very top of the bundle via output.intro.
// Uses top-level await + dynamic import so they execute before ANY
// bundled code (including WASM init which needs fetch(file://...)).
const nodePolyfills = `
const { readFileSync: __nReadFileSync } = await import('node:fs');
const { fileURLToPath: __nFileURLToPath } = await import('node:url');
const { Agent: __nAgent, setGlobalDispatcher: __nSetGlobalDispatcher } = await import('undici');

__nSetGlobalDispatcher(new __nAgent({ allowH2: true }));

const __nOriginalFetch = globalThis.fetch;
globalThis.fetch = async function __patchedNodeFetch(input, init) {
  let url;
  if (input instanceof URL) url = input;
  else if (input instanceof Request) url = new URL(input.url);
  else if (typeof input === 'string') {
    try { url = new URL(input); } catch { return __nOriginalFetch(input, init); }
  }
  if (url && url.protocol === 'file:') {
    const buffer = __nReadFileSync(__nFileURLToPath(url));
    return new Response(buffer, { status: 200, headers: { 'Content-Type': 'application/wasm' } });
  }
  return __nOriginalFetch(input, init);
};

if (typeof globalThis.self === 'undefined') { globalThis.self = globalThis; }
`;

export default [
  {
    input: ["./js/node-entry.js"],
    output: {
      dir: "dist-node",
      format: "es",
      sourcemap: true,
      assetFileNames: "assets/[name][extname]",
      intro: nodePolyfills,
    },
    external: ["better-sqlite3", "undici", /^node:/],
    plugins: [
      rust({
        verbose: true,
        extraArgs: {
          cargo: [...baseCargoArgs],
          wasmOpt: wasmOptArgs,
          wasmBindgen: devMode ? ["--keep-debug"] : [],
        },
        experimental: {
          typescriptDeclarationDir: "dist-node/crates",
        },
        optimize: { release: true, rustc: !devMode },
      }),
      resolve(),
      commonjs(),
    ],
  },
];
