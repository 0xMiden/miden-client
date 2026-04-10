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
  fullyParallel: process.env.TEST_MIDEN_PROVER_URL ? false : true,
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  /* Retry on CI only */
  retries: 0,
  /* Opt out of parallel tests on CI. */
  workers: process.env.CI ? 2 : undefined,
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
        // Node.js-only tests (use napi-specific JS wrapper)
        "test/compile_and_contract*",
        "test/miden_client_api*",
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
        // Node.js-only tests (use napi-specific JS wrapper)
        "test/compile_and_contract*",
        "test/miden_client_api*",
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
      // Skip specific browser-only tests by name.
      // Tests that request the `page` fixture must be listed here because
      // Playwright launches the browser for the fixture BEFORE test.skip()
      // in the test body can run.
      grepInvert:
        /exportStore|importStore|reads updated state after a mutating|accounts\.insert stores a pre-built/,
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
          command: "npx http-server ./dist -a localhost -p 8080",
          url: "http://localhost:8080",
          reuseExistingServer: true,
        },
      }),
});
// CI trigger
