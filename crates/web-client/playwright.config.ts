import { defineConfig, devices } from "@playwright/test";

/**
 * Read environment variables from file.
 * https://github.com/motdotla/dotenv
 */
// import dotenv from 'dotenv';
// import path from 'path';
// dotenv.config({ path: path.resolve(__dirname, '.env') });

/**
 * See https://playwright.dev/docs/test-configuration.
 */

export default defineConfig({
  timeout: 240_000,
  testDir: "./test",
  /* Run tests in files in parallel */
  fullyParallel: process.env.REMOTE_PROVER ? false : true,
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  /* Retry on CI only */
  retries: 0,
  /* Opt out of parallel tests on CI. */
  workers: process.env.CI ? 1 : undefined,
  /* Reporter to use. See https://playwright.dev/docs/test-reporters */
  reporter: "html",
  /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
  use: {
    /* Base URL to use in actions like `await page.goto('/')`. */
    // baseURL: 'http://localhost:3000',

    /* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
    trace: "on-first-retry",
  },

  /* Configure projects for major browsers */
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
      testMatch: "*.test.ts",
      testIgnore: [
        "test/node/**",
        "test/shared/**",
        // Ported files use test-setup.ts fixtures (Node.js only for now)
        "test/account_component*",
        "test/account_file*",
        "test/account_reader*",
        "test/account.test*",
        "test/address*",
        "test/basic_fungible_faucet*",
        "test/compile_and_contract*",
        "test/fpi*",
        "test/import.test*",
        "test/key*",
        "test/miden_client_api*",
        "test/mockchain*",
        "test/multisig*",
        "test/new_account*",
        "test/new_transactions*",
        "test/note_transport*",
        "test/notes*",
        "test/settings*",
        "test/swap*",
        "test/tags*",
        "test/token_symbol*",
        "test/transactions*",
      ],
    },

    // {
    //   name: "firefox",
    //   use: { ...devices["Desktop Firefox"] },
    // },

    {
      name: "webkit",
      use: { ...devices["Desktop Safari"] },
      testIgnore: [
        "test/node/**",
        "test/shared/**",
        // Same as chromium — ported files are Node.js only
        "test/account_component*",
        "test/account_file*",
        "test/account_reader*",
        "test/account.test*",
        "test/address*",
        "test/basic_fungible_faucet*",
        "test/compile_and_contract*",
        "test/fpi*",
        "test/import.test*",
        "test/key*",
        "test/miden_client_api*",
        "test/mockchain*",
        "test/multisig*",
        "test/new_account*",
        "test/new_transactions*",
        "test/note_transport*",
        "test/notes*",
        "test/settings*",
        "test/swap*",
        "test/tags*",
        "test/token_symbol*",
        "test/transactions*",
      ],
    },

    {
      name: "nodejs",
      testDir: "./test",
      testMatch: "**/*.test.ts",
      // Skip browser-only and WASM-specific tests
      testIgnore: [
        "test/store_isolation*",
        "test/sync_lock*",
        "test/import_export*",
        "test/remote_keystore*",
        "test/package*", // TestUtils (createMockSerialized*) is browser-only
        "test/miden_array*", // WASM array .length() method not available in Node.js
        "test/shared/**", // Old format duplicates (ported to root test/)
        "test/node/**", // Old format duplicates (ported to root test/)
        "test/remote_prover_transactions*", // Old browser format for chromium CI
      ],
      // Skip specific browser-only tests by name
      grepInvert: /exportStore|importStore/,
    },

    /* Test against mobile viewports. */
    // {
    //   name: 'Mobile Chrome',
    //   use: { ...devices['Pixel 5'] },
    // },
    // {
    //   name: 'Mobile Safari',
    //   use: { ...devices['iPhone 12'] },
    // },

    /* Test against branded browsers. */
    // {
    //   name: 'Microsoft Edge',
    //   use: { ...devices['Desktop Edge'], channel: 'msedge' },
    // },
    // {
    //   name: 'Google Chrome',
    //   use: { ...devices['Desktop Chrome'], channel: 'chrome' },
    // },
  ],

  /* Run your local dev server before starting the tests */
  // FIXME: Modularise test server constants (localhost, port)
  // Skip webServer when running only Node.js tests (no browser/dist needed)
  ...(process.env.SKIP_WEB_SERVER
    ? {}
    : {
        webServer: {
          command: "npx http-server ./dist -p 8080",
          url: "http://127.0.0.1:8080",
          reuseExistingServer: true,
        },
      }),
});
