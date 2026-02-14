import { defineConfig } from "vitest/config";
import { fileURLToPath } from "url";

process.env.VITE_CJS_IGNORE_WARNING = "1";

export default defineConfig({
  resolve: {
    alias: {
      "@miden-sdk/miden-sdk": fileURLToPath(
        new URL("./src/__tests__/mocks/miden-sdk-entry.ts", import.meta.url)
      ),
    },
  },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/__tests__/setup.ts"],
    include: ["src/__tests__/**/*.test.{ts,tsx}"],
    coverage: {
      provider: "v8",
      reporter: ["text", "json", "html"],
      include: ["src/**/*.{ts,tsx}"],
      exclude: ["src/__tests__/**", "src/index.ts"],
    },
  },
});
