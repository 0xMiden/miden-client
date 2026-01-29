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
  workers: process.env.CI ? 1 : undefined,
  reporter: "html",

  use: {
    trace: "on-first-retry",
  },

  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
      testMatch: "*.test.ts",
    },
    {
      name: "webkit",
      use: { ...devices["Desktop Safari"] },
      testMatch: "*.test.ts",
    },
  ],

  webServer: {
    command: "node ./test/serve-tests.js",
    url: "http://127.0.0.1:8081",
    reuseExistingServer: true,
    timeout: 30000,
  },
});
