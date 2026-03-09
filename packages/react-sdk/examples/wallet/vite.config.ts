import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { midenVitePlugin } from "@miden-sdk/vite-plugin";
import path from "path";

// Published packages: import { paraVitePlugin } from "@miden-sdk/use-miden-para-react/vite";
import { paraVitePlugin } from "../../../../../miden-para/packages/use-miden-para-react/src/paraVitePlugin";

export default defineConfig({
  plugins: [
    react(),
    midenVitePlugin(),
    paraVitePlugin(),
  ],
  resolve: {
    alias: {
      // Local dev overrides — not needed when using published packages
      "@miden-sdk/react": path.resolve(__dirname, "../../src/index.ts"),
      "@miden-sdk/miden-turnkey-react": path.resolve(__dirname, "../../../../../miden-turnkey/packages/use-miden-turnkey-react/dist/index.mjs"),
    },
  },
});
