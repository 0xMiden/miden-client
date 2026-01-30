import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  optimizeDeps: {
    exclude: ["@miden-sdk/miden-sdk"],
  },
  resolve: {
    dedupe: ["react", "react-dom", "react/jsx-runtime"],
  },
});
