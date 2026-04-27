import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright configuration for React SDK integration tests.
 * These tests run in a real browser with the actual WASM SDK and MockWebClient.
 */
export default defineConfig({
  timeout: 120_000,
  testDir: "./test",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: 0,
  workers: process.env.CI ? 2 : undefined,
  reporter: "html",

  use: {
    trace: "on-first-retry",
  },

  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
      testMatch: "*.test.ts",
      // MockWebClient is not yet compatible with the js_export architecture
      testIgnore: ["test/mockchain*"],
    },
    {
      name: "webkit",
      use: { ...devices["Desktop Safari"] },
      testMatch: "*.test.ts",
      testIgnore: ["test/mockchain*"],
    },
  ],

  webServer: {
    command: "node ./test/serve-tests.js",
    url: "http://127.0.0.1:8081",
    reuseExistingServer: true,
    timeout: 30000,
  },
});
