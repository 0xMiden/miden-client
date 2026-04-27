import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { midenVitePlugin } from "@miden-sdk/vite-plugin";
import { paraVitePlugin } from "@miden-sdk/use-miden-para-react/vite";

// Production-shaped config — what a normal consumer would write.
//   - midenVitePlugin: WASM dedupe + worker-context node polyfills.
//   - paraVitePlugin:  Para optional-deps stubs + main-context polyfills
//                      + rollup externalization.
//
// Local dev linking is done entirely through `file:` deps in package.json
// (both `dependencies` and `resolutions` so transitive peerDeps land on the
// same copy). To pick up local edits to a sibling package: rebuild that
// package, then `rm -rf node_modules && yarn install` here — yarn1 file:
// deps are copies, not symlinks.
export default defineConfig({
  plugins: [react(), midenVitePlugin(), paraVitePlugin()],
});
