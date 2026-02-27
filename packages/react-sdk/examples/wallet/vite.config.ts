import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { midenVitePlugin } from "@miden-sdk/vite-plugin";

export default defineConfig({
  plugins: [react(), midenVitePlugin({ rpcProxyTarget: false })],
  resolve: {
    dedupe: ["react", "react-dom", "react/jsx-runtime"],
  },
});
