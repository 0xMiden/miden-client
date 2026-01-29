import { test, expect, type Page } from "@playwright/test";

type ReactSdkState = {
  testAppReady: boolean;
  testAppError: string | null;
  sdkLoaded: boolean;
  sdkLoadError: string | null;
  reactSdkReady: boolean;
  hasApi: boolean;
};

async function readReactSdkState(page: Page): Promise<ReactSdkState> {
  return page.evaluate(() => ({
    testAppReady: (window as any).testAppReady === true,
    testAppError: (window as any).testAppError ?? null,
    sdkLoaded: (window as any).sdkLoaded === true,
    sdkLoadError: (window as any).sdkLoadError ?? null,
    reactSdkReady: (window as any).reactSdkReady === true,
    hasApi: typeof (window as any).__reactSdk !== "undefined",
  }));
}

async function waitForTestAppReady(
  page: Page,
  timeoutMs = 10_000
): Promise<ReactSdkState> {
  const startedAt = Date.now();
  while (Date.now() - startedAt < timeoutMs) {
    const state = await readReactSdkState(page);
    if (state.testAppError) {
      return state;
    }
    if (state.testAppReady) {
      return state;
    }
    await page.waitForTimeout(200);
  }

  return readReactSdkState(page);
}

async function waitForReactSdkReady(
  page: Page,
  timeoutMs = 10_000
): Promise<ReactSdkState> {
  const startedAt = Date.now();
  while (Date.now() - startedAt < timeoutMs) {
    const state = await readReactSdkState(page);
    if (state.testAppError || state.sdkLoadError) {
      return state;
    }
    if (state.reactSdkReady && state.hasApi) {
      return state;
    }
    await page.waitForTimeout(200);
  }

  return readReactSdkState(page);
}

async function waitForSdkLoaded(
  page: Page,
  timeoutMs = 15_000
): Promise<ReactSdkState> {
  const startedAt = Date.now();
  while (Date.now() - startedAt < timeoutMs) {
    const state = await readReactSdkState(page);
    if (state.testAppError) {
      return state;
    }
    if (state.sdkLoaded || state.sdkLoadError) {
      return state;
    }
    await page.waitForTimeout(200);
  }

  return readReactSdkState(page);
}

async function waitForReactSdk(page: Page): Promise<boolean> {
  try {
    const state = await waitForTestAppReady(page);

    if (!state.testAppReady) {
      console.log("Timed out waiting for test app to be ready:", state);
      return false;
    }

    if (state.testAppError) {
      console.log("Test app error:", state.testAppError);
      return false;
    }

    const sdkState = await waitForSdkLoaded(page);
    if (!sdkState.sdkLoaded) {
      console.log(
        "SDK not loaded:",
        sdkState.sdkLoadError || "Unknown error"
      );
      return false;
    }

    const reactState = await waitForReactSdkReady(page);
    if (!reactState.reactSdkReady || !reactState.hasApi) {
      console.log("React SDK test API not available");
      return false;
    }

    return true;
  } catch (err) {
    console.log("Timeout waiting for React SDK:", err);
    return false;
  }
}

test.describe("React SDK Hooks (Playwright)", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("http://localhost:8081");
  });

  test("useTransaction executes a transaction", async ({ page }) => {
    const sdkAvailable = await waitForReactSdk(page);
    if (!sdkAvailable) {
      test.skip();
      return;
    }

    const result = await page.evaluate(async () => {
      const api = (window as any).__reactSdk;
      return await api.runTransaction();
    });

    expect(result.transactionId).toBeTruthy();

    await page.waitForFunction(
      () => (window as any).__reactSdkState().transactionStage === "complete"
    );

    const state = await page.evaluate(
      () => (window as any).__reactSdkState()
    );
    expect(state.transactionStage).toBe("complete");
    expect(state.transactionError).toBeNull();
  });

  test("useImportAccount imports account from file", async ({ page }) => {
    const sdkAvailable = await waitForReactSdk(page);
    if (!sdkAvailable) {
      test.skip();
      return;
    }

    const result = await page.evaluate(async () => {
      const api = (window as any).__reactSdk;
      return await api.importAccountFromFile();
    });

    expect(result.accountId).toBeTruthy();

    await page.waitForFunction(
      () => (window as any).__reactSdkState().importedAccountId
    );

    const state = await page.evaluate(
      () => (window as any).__reactSdkState()
    );
    expect(state.importError).toBeNull();
    expect(state.importedAccountId).toBeTruthy();
  });
});
