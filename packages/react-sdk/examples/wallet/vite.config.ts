import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { midenVitePlugin } from "@miden-sdk/vite-plugin";
import { paraVitePlugin } from "@miden-sdk/use-miden-para-react/vite";
import path from "path";

export default defineConfig({
  plugins: [react(), midenVitePlugin(), paraVitePlugin()],
  resolve: {
    alias: {
      // Use local source for react-sdk development
      "@miden-sdk/react": path.resolve(__dirname, "../../src/index.ts"),
    },
  },
});
