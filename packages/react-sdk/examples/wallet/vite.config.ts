import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { midenVitePlugin } from "@miden-sdk/vite-plugin";
import { paraVitePlugin } from "@miden-sdk/use-miden-para-react/vite";
import path from "path";

// ─────────────────────────────────────────────────────────────────────
// Production-shaped config: anything a normal consumer would write.
// midenVitePlugin handles WASM dedupe + worker-context node polyfills.
// paraVitePlugin handles Para's optional-deps stubs + main-context
// polyfills + rollup externalization.
// ─────────────────────────────────────────────────────────────────────
const productionConfig = {
  plugins: [react(), midenVitePlugin(), paraVitePlugin()],
};

// ─────────────────────────────────────────────────────────────────────
// LOCAL DEV ONLY — do NOT copy these into a real consumer's vite.config.
// They exist because this example consumes the SDK + signer adapters via
// `file:` deps to sibling repos. Remove all of this when consuming the
// published packages.
// ─────────────────────────────────────────────────────────────────────
const localDevOverrides = {
  resolve: {
    alias: [
      // SDK source — for instant HMR on react-sdk edits.
      {
        find: "@miden-sdk/react",
        replacement: path.resolve(__dirname, "../../src/index.ts"),
      },
      // SDK uses `resolveAuthScheme` which only exists in the local
      // crates/web-client build (HEAD); registry @miden-sdk/miden-sdk@0.14.4
      // doesn't export it. Aliasing both bare + /lazy specifiers.
      {
        find: /^@miden-sdk\/miden-sdk\/lazy$/,
        replacement: path.resolve(
          __dirname,
          "../../../../crates/web-client/dist/index.js"
        ),
      },
      {
        find: /^@miden-sdk\/miden-sdk$/,
        replacement: path.resolve(
          __dirname,
          "../../../../crates/web-client/dist/eager.js"
        ),
      },
    ],
  },
  server: {
    fs: {
      // Vite restricts /@fs/ to the project root by default; widen so
      // local file: deps and the local crates/web-client WASM can be
      // served.
      allow: [path.resolve(__dirname, "../../../..")],
    },
  },
};

export default defineConfig({
  ...productionConfig,
  ...localDevOverrides,
});
