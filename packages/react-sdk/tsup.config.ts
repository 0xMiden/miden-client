import { defineConfig } from "tsup";
import { readFileSync, writeFileSync } from "fs";
import { join } from "path";

/**
 * Post-build rewrite: swap every `@miden-sdk/miden-sdk/lazy` import in the
 * eager bundles (`index.{js,mjs}`) to `@miden-sdk/miden-sdk` (the default
 * entry). Consumer bundlers resolve the rewritten string to the SDK's eager
 * variant, which initializes WASM via TLA on import.
 *
 * The React SDK's source tree always imports from the `/lazy` subpath, so the
 * lazy build (`lazy.{js,mjs}`) ships unchanged. We rewrite only the eager
 * bundles at the emitted-file level, which is more reliable than an esbuild
 * `onResolve` hook — tsup's default externalization from `peerDependencies`
 * happens before our plugin gets a chance to change the import path.
 */
function rewriteEagerBundles(distDir: string): void {
  for (const file of ["index.js", "index.mjs"]) {
    const path = join(distDir, file);
    const before = readFileSync(path, "utf8");
    const after = before.replace(
      /@miden-sdk\/miden-sdk\/lazy/g,
      "@miden-sdk/miden-sdk"
    );
    if (after === before) continue;
    writeFileSync(path, after);
  }
}

export default defineConfig([
  // Eager variant — default entry (`@miden-sdk/react`).
  //
  // Source imports `@miden-sdk/miden-sdk/lazy`; `onSuccess` rewrites those
  // to `@miden-sdk/miden-sdk` after emit, so consumer bundlers resolve
  // against the SDK's eager default.
  {
    entry: { index: "src/index.ts" },
    format: ["cjs", "esm"],
    dts: true,
    clean: true,
    onSuccess: async () => {
      rewriteEagerBundles("dist");
    },
  },
  // Lazy variant — subpath entry (`@miden-sdk/react/lazy`).
  //
  // No rewrite; imports keep `@miden-sdk/miden-sdk/lazy` so consumer
  // bundlers resolve them against the SDK's lazy subpath (no TLA).
  // Required for Capacitor hosts, Next.js SSR, and any environment that
  // can't tolerate top-level await at SDK module evaluation.
  {
    entry: { lazy: "src/index.ts" },
    format: ["cjs", "esm"],
    dts: true,
    clean: false,
  },
]);
