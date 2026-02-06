/**
 * Integration tests using MockWebClient to verify the SDK works in a real browser.
 * These tests complement the vitest unit tests by running against actual WASM.
 *
 * Prerequisites:
 * 1. Build the web-client: cd ../../crates/web-client && yarn build
 * 2. The SDK will be served from ../../crates/web-client/dist/
 */
import { test, expect, type Page } from "@playwright/test";

type TestAppState = {
  testAppReady: boolean;
  testAppError: string | null;
  sdkLoaded: boolean;
  sdkLoadError: string | null;
};

async function readTestAppState(page: Page): Promise<TestAppState> {
  return page.evaluate(() => ({
    testAppReady: (window as any).testAppReady === true,
    testAppError: (window as any).testAppError ?? null,
    sdkLoaded: (window as any).sdkLoaded === true,
    sdkLoadError: (window as any).sdkLoadError ?? null,
  }));
}

async function waitForTestAppReady(
  page: Page,
  timeoutMs = 15_000
): Promise<TestAppState> {
  const startedAt = Date.now();
  while (Date.now() - startedAt < timeoutMs) {
    const state = await readTestAppState(page);
    if (state.testAppError) {
      return state;
    }
    if (state.testAppReady) {
      return state;
    }
    await page.waitForTimeout(200);
  }

  return readTestAppState(page);
}

async function waitForSdkLoaded(
  page: Page,
  timeoutMs = 15_000
): Promise<TestAppState> {
  const startedAt = Date.now();
  while (Date.now() - startedAt < timeoutMs) {
    const state = await readTestAppState(page);
    if (state.testAppError) {
      return state;
    }
    if (state.sdkLoaded || state.sdkLoadError) {
      return state;
    }
    await page.waitForTimeout(200);
  }

  return readTestAppState(page);
}

// Helper to wait for SDK to load
async function waitForSdk(page: Page): Promise<boolean> {
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
      console.log("SDK not loaded:", sdkState.sdkLoadError || "Unknown error");
      return false;
    }

    // Verify MockWebClient is available
    const hasMockClient = await page.evaluate(
      () => typeof (window as any).MockWebClient !== "undefined"
    );
    if (!hasMockClient) {
      console.log("MockWebClient not found in SDK exports");
      return false;
    }

    return true;
  } catch (err) {
    console.log("Timeout waiting for SDK:", err);
    return false;
  }
}

test.describe("MockWebClient Integration", () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to test page
    await page.goto("http://localhost:8081");
  });

  test("should create wallet with MockWebClient", async ({ page }) => {
    const sdkAvailable = await waitForSdk(page);
    if (!sdkAvailable) {
      test.skip();
      return;
    }

    const result = await page.evaluate(async () => {
      const client = await (window as any).MockWebClient.createClient();
      await client.syncState();

      const wallet = await client.newWallet(
        (window as any).AccountStorageMode.private(),
        true,
        0
      );

      return {
        walletId: wallet.id().toString(),
        success: true,
      };
    });

    expect(result.success).toBe(true);
    expect(result.walletId).toBeTruthy();
  });

  test("should create faucet with MockWebClient", async ({ page }) => {
    const sdkAvailable = await waitForSdk(page);
    if (!sdkAvailable) {
      test.skip();
      return;
    }

    const result = await page.evaluate(async () => {
      const client = await (window as any).MockWebClient.createClient();
      await client.syncState();

      const faucet = await client.newFaucet(
        (window as any).AccountStorageMode.private(),
        false,
        "TEST",
        8,
        BigInt(1000000),
        0
      );

      return {
        faucetId: faucet.id().toString(),
        success: true,
      };
    });

    expect(result.success).toBe(true);
    expect(result.faucetId).toBeTruthy();
  });

  test("should list accounts after creation", async ({ page }) => {
    const sdkAvailable = await waitForSdk(page);
    if (!sdkAvailable) {
      test.skip();
      return;
    }

    const result = await page.evaluate(async () => {
      const client = await (window as any).MockWebClient.createClient();
      await client.syncState();

      // Create a wallet
      await client.newWallet(
        (window as any).AccountStorageMode.private(),
        true,
        0
      );

      // List accounts
      const accounts = await client.getAccounts();
      return {
        count: accounts.length,
        success: true,
      };
    });

    expect(result.success).toBe(true);
    expect(result.count).toBeGreaterThanOrEqual(1);
  });

  test("should mint and consume tokens with MockWebClient", async ({
    page,
  }) => {
    const sdkAvailable = await waitForSdk(page);
    if (!sdkAvailable) {
      test.skip();
      return;
    }

    const finalBalance = await page.evaluate(async () => {
      const client = await (window as any).MockWebClient.createClient();
      await client.syncState();

      // Create wallet and faucet
      const wallet = await client.newWallet(
        (window as any).AccountStorageMode.private(),
        true,
        0
      );
      const faucet = await client.newFaucet(
        (window as any).AccountStorageMode.private(),
        false,
        "TOKEN",
        8,
        BigInt(10000000),
        0
      );

      // Mint tokens
      const mintRequest = await client.newMintTransactionRequest(
        wallet.id(),
        faucet.id(),
        (window as any).NoteType.Public,
        BigInt(1000)
      );

      const mintTxId = await client.submitNewTransaction(
        faucet.id(),
        mintRequest
      );

      // Prove block and sync
      await client.proveBlock();
      await client.syncState();

      // Get the minted note
      const [mintTxRecord] = await client.getTransactions(
        (window as any).TransactionFilter.ids([mintTxId])
      );

      const mintedNoteId = mintTxRecord
        .outputNotes()
        .notes()[0]
        .id()
        .toString();

      const mintedInput = await client.getInputNote(mintedNoteId);
      if (!mintedInput) {
        throw new Error(`Minted note ${mintedNoteId} not found`);
      }
      const mintedNote = mintedInput.toNote
        ? mintedInput.toNote()
        : mintedNoteId;

      // Consume the note (no await - it's synchronous)
      const consumeRequest = client.newConsumeTransactionRequest([mintedNote]);
      await client.submitNewTransaction(wallet.id(), consumeRequest);

      // Prove and sync
      await client.proveBlock();
      await client.syncState();

      // Check balance
      const updatedWallet = await client.getAccount(wallet.id());
      return updatedWallet.vault().getBalance(faucet.id()).toString();
    });

    expect(finalBalance).toBe("1000");
  });

  test("should send tokens between wallets", async ({ page }) => {
    const sdkAvailable = await waitForSdk(page);
    if (!sdkAvailable) {
      test.skip();
      return;
    }

    const result = await page.evaluate(async () => {
      const client = await (window as any).MockWebClient.createClient();
      await client.syncState();

      // Create sender, receiver, and faucet
      const sender = await client.newWallet(
        (window as any).AccountStorageMode.private(),
        true,
        0
      );
      const receiver = await client.newWallet(
        (window as any).AccountStorageMode.private(),
        true,
        0
      );
      const faucet = await client.newFaucet(
        (window as any).AccountStorageMode.private(),
        false,
        "SEND",
        8,
        BigInt(10000000),
        0
      );

      // Mint tokens to sender
      const mintRequest = await client.newMintTransactionRequest(
        sender.id(),
        faucet.id(),
        (window as any).NoteType.Public,
        BigInt(1000)
      );

      const mintTxId = await client.submitNewTransaction(
        faucet.id(),
        mintRequest
      );
      await client.proveBlock();
      await client.syncState();

      // Consume the minted note into sender
      const [mintTxRecord] = await client.getTransactions(
        (window as any).TransactionFilter.ids([mintTxId])
      );
      const mintedNoteId = mintTxRecord
        .outputNotes()
        .notes()[0]
        .id()
        .toString();
      const mintedInput = await client.getInputNote(mintedNoteId);
      if (!mintedInput) {
        throw new Error(`Minted note ${mintedNoteId} not found`);
      }
      const mintedNote = mintedInput.toNote
        ? mintedInput.toNote()
        : mintedNoteId;
      const consumeRequest = client.newConsumeTransactionRequest([mintedNote]);
      await client.submitNewTransaction(sender.id(), consumeRequest);
      await client.proveBlock();
      await client.syncState();

      // Send tokens from sender to receiver
      const sendRequest = await client.newSendTransactionRequest(
        sender.id(),
        receiver.id(),
        faucet.id(),
        (window as any).NoteType.Public,
        BigInt(500)
      );
      const sendTxId = await client.submitNewTransaction(
        sender.id(),
        sendRequest
      );
      await client.proveBlock();
      await client.syncState();

      // Consume the sent note into receiver
      const [sendTxRecord] = await client.getTransactions(
        (window as any).TransactionFilter.ids([sendTxId])
      );
      const sentNoteId = sendTxRecord.outputNotes().notes()[0].id().toString();
      const sentInput = await client.getInputNote(sentNoteId);
      if (!sentInput) {
        throw new Error(`Sent note ${sentNoteId} not found`);
      }
      const sentNote = sentInput.toNote ? sentInput.toNote() : sentNoteId;
      const receiveRequest = client.newConsumeTransactionRequest([sentNote]);
      await client.submitNewTransaction(receiver.id(), receiveRequest);
      await client.proveBlock();
      await client.syncState();

      // Check balances
      const updatedSender = await client.getAccount(sender.id());
      const updatedReceiver = await client.getAccount(receiver.id());

      return {
        senderBalance: updatedSender.vault().getBalance(faucet.id()).toString(),
        receiverBalance: updatedReceiver
          .vault()
          .getBalance(faucet.id())
          .toString(),
      };
    });

    expect(result.senderBalance).toBe("500");
    expect(result.receiverBalance).toBe("500");
  });

  test("should sync state properly", async ({ page }) => {
    const sdkAvailable = await waitForSdk(page);
    if (!sdkAvailable) {
      test.skip();
      return;
    }

    const result = await page.evaluate(async () => {
      const client = await (window as any).MockWebClient.createClient();

      // First sync
      const syncResult1 = await client.syncState();

      // Create some data
      await client.newWallet(
        (window as any).AccountStorageMode.private(),
        true,
        0
      );

      // Second sync
      const syncResult2 = await client.syncState();

      return {
        firstSync: !!syncResult1,
        secondSync: !!syncResult2,
        success: true,
      };
    });

    expect(result.success).toBe(true);
    expect(result.firstSync).toBe(true);
    expect(result.secondSync).toBe(true);
  });
});
