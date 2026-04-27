// @ts-nocheck
import test from "./playwright.global.setup";
import { Page, expect } from "@playwright/test";
import {
  setupWalletAndFaucet,
  mintTransaction,
  consumeTransaction,
  sendTransaction,
  setupMintedNote,
  getSyncHeight,
  mintAndConsumeTransaction,
  swapTransaction,
} from "./webClientTestUtils";
import { ConsumableNoteRecord } from "../dist/crates/miden_client_web";

const getConsumableNotes = async (
  testingPage: Page,
  accountId?: string
): Promise<
  {
    noteId: string;
    consumability: {
      accountId: string;
      consumableAfterBlock: number | undefined;
    }[];
  }[]
> => {
  return await testingPage.evaluate(async (_accountId?: string) => {
    const client = window.client;
    let records;
    if (_accountId) {
      const accountId = window.AccountId.fromHex(_accountId);
      records = await client.getConsumableNotes(accountId);
    } else {
      records = await client.getConsumableNotes();
    }

    return records.map((record: ConsumableNoteRecord) => ({
      noteId: record.inputNoteRecord().id().toString(),
      consumability: record.noteConsumability().map((c) => ({
        accountId: c.accountId().toString(),
        consumableAfterBlock: c.consumptionStatus()?.consumableAfterBlock(),
      })),
    }));
  }, accountId);
};

// Remote prover transaction tests.
// These re-run key transaction flows using a remote prover.
// They require a running node + remote prover service (REMOTE_PROVER env).
// The CI grep filter matches test names containing "with remote prover".

test.describe("remote prover transaction tests", () => {
  test.skip(
    !process.env.TEST_MIDEN_PROVER_URL,
    "requires a running remote prover service (TEST_MIDEN_PROVER_URL)"
  );

  test("mint transaction with remote prover completes successfully", async ({
    page,
  }) => {
    const { accountId, faucetId } = await setupWalletAndFaucet(page);
    const result = await mintTransaction(page, accountId, faucetId, true, true);
    expect(result.numOutputNotesCreated).toEqual(1);
    expect(result.createdNoteId).toBeDefined();
  });

  test("consume transaction with remote prover completes successfully", async ({
    page,
  }) => {
    const { accountId, faucetId } = await setupWalletAndFaucet(page);
    const { createdNoteId } = await mintTransaction(
      page,
      accountId,
      faucetId,
      true,
      true
    );
    const result = await consumeTransaction(
      page,
      accountId,
      faucetId,
      createdNoteId,
      true
    );
    expect(result.targetAccountBalance).toEqual("1000");
  });

  test("send transaction with remote prover completes successfully", async ({
    page,
  }) => {
    const { accountId: senderId, faucetId } = await setupWalletAndFaucet(page);
    const { accountId: targetId } = await setupWalletAndFaucet(page);
    const result = await sendTransaction(
      page,
      senderId,
      targetId,
      faucetId,
      undefined,
      true
    );
    expect(result.length).toBeGreaterThanOrEqual(1);
  });

  test("custom transaction with remote prover completes successfully", async ({
    page,
  }) => {
    await page.evaluate(async () => {
      const client = window.client;
      await client.syncState();

      const wallet = await client.newWallet(
        window.AccountStorageMode.private(),
        false,
        window.AuthScheme.AuthRpoFalcon512
      );

      const txScript = `
        begin
          push.0 push.0
          assert_eq
        end
      `;

      const builder = await client.createCodeBuilder();
      const transactionScript = builder.compileTxScript(txScript);

      const transactionRequest = new window.TransactionRequestBuilder()
        .withCustomScript(transactionScript)
        .build();

      const prover =
        window.remoteProverUrl != null
          ? window.TransactionProver.newRemoteProver(
              window.remoteProverUrl,
              null
            )
          : undefined;

      await window.helpers.executeAndApplyTransaction(
        wallet.id(),
        transactionRequest,
        prover
      );
    });
  });

  test("no filter by account with remote prover", async ({ page }) => {
    test.slow();
    const { createdNoteId: noteId1, accountId: accountId1 } =
      await setupMintedNote(page, false, true);
    const { createdNoteId: noteId2, accountId: accountId2 } =
      await setupMintedNote(page, false, true);

    const noteIds = new Set([noteId1, noteId2]);
    const accountIds = new Set([accountId1, accountId2]);
    const result = await getConsumableNotes(page);
    expect(noteIds).toEqual(new Set(result.map((r) => r.noteId)));
    expect(accountIds).toEqual(
      new Set(result.map((r) => r.consumability[0].accountId))
    );
    expect(result.length).toBeGreaterThanOrEqual(2);
    const consumableRecord1 = result.find((r) => r.noteId === noteId1);
    const consumableRecord2 = result.find((r) => r.noteId === noteId2);

    consumableRecord1!!.consumability.forEach((c) => {
      expect(c.accountId).toEqual(accountId1);
    });

    consumableRecord2!!.consumability.forEach((c) => {
      expect(c.accountId).toEqual(accountId2);
    });
  });

  test("p2ide consume after block with remote prover", async ({ page }) => {
    test.slow();
    const RECALL_HEIGHT_DELTA = 500;
    const { accountId: senderAccountId, faucetId } =
      await setupWalletAndFaucet(page);
    const { accountId: targetAccountId } = await setupWalletAndFaucet(page);
    const recallHeight = (await getSyncHeight(page)) + RECALL_HEIGHT_DELTA;
    await sendTransaction(
      page,
      senderAccountId,
      targetAccountId,
      faucetId,
      recallHeight,
      true
    );

    const consumableRecipient = await getConsumableNotes(page, targetAccountId);
    const consumableSender = await getConsumableNotes(page, senderAccountId);
    expect(consumableSender.length).toBe(1);
    expect(consumableSender[0].consumability[0].consumableAfterBlock).toBe(
      recallHeight
    );
    expect(consumableRecipient.length).toBe(1);
    expect(
      consumableRecipient[0].consumability[0].consumableAfterBlock
    ).toBeUndefined();
  });

  test("swap transaction with remote prover completes successfully", async ({
    page,
  }) => {
    test.setTimeout(900000);
    const { accountId: accountA, faucetId: faucetA } =
      await setupWalletAndFaucet(page);
    const { accountId: accountB, faucetId: faucetB } =
      await setupWalletAndFaucet(page);

    const assetAAmount = BigInt(1);
    const assetBAmount = BigInt(25);

    await mintAndConsumeTransaction(page, accountA, faucetA, true);
    await mintAndConsumeTransaction(page, accountB, faucetB, true);

    const { accountAAssets, accountBAssets } = await swapTransaction(
      page,
      accountA,
      accountB,
      faucetA,
      assetAAmount,
      faucetB,
      assetBAmount,
      "private",
      "private",
      true
    );

    // --- assertions for Account A ---
    const aA = accountAAssets!.find((a) => a.assetId === faucetA);
    expect(aA, `Expected to find asset ${faucetA} on Account A`).toBeTruthy();
    expect(BigInt(aA!.amount)).toEqual(999n);

    const aB = accountAAssets!.find((a) => a.assetId === faucetB);
    expect(aB, `Expected to find asset ${faucetB} on Account A`).toBeTruthy();
    expect(BigInt(aB!.amount)).toEqual(25n);

    // --- assertions for Account B ---
    const bA = accountBAssets!.find((a) => a.assetId === faucetA);
    expect(bA, `Expected to find asset ${faucetA} on Account B`).toBeTruthy();
    expect(BigInt(bA!.amount)).toEqual(1n);

    const bB = accountBAssets!.find((a) => a.assetId === faucetB);
    expect(bB, `Expected to find asset ${faucetB} on Account B`).toBeTruthy();
    expect(BigInt(bB!.amount)).toEqual(975n);
  });
});
