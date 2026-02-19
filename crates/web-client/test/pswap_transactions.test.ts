import test from "./playwright.global.setup";
import { expect } from "@playwright/test";
import {
  mintAndConsumeTransaction,
  setupWalletAndFaucet,
} from "./webClientTestUtils";

// PSWAP_TRANSACTION TESTS
// =======================================================================================================

test.describe("pswap transaction tests", () => {
  test("pswap create and consume round-trip completes successfully", async ({
    page,
  }) => {
    test.setTimeout(480000);

    const { accountId: accountA, faucetId: faucetA } =
      await setupWalletAndFaucet(page);
    const { accountId: accountB, faucetId: faucetB } =
      await setupWalletAndFaucet(page);

    await mintAndConsumeTransaction(page, accountA, faucetA, false);
    await mintAndConsumeTransaction(page, accountB, faucetB, false);

    const result = await page.evaluate(
      async ({
        _accountAId,
        _accountBId,
        _faucetAId,
        _faucetBId,
      }: {
        _accountAId: string;
        _accountBId: string;
        _faucetAId: string;
        _faucetBId: string;
      }) => {
        const client = window.client;

        await client.syncState();

        const accountAId = window.AccountId.fromHex(_accountAId);
        const accountBId = window.AccountId.fromHex(_accountBId);
        const faucetAId = window.AccountId.fromHex(_faucetAId);
        const faucetBId = window.AccountId.fromHex(_faucetBId);

        // Create PSWAP note: account A offers 100 of asset A for 50 of asset B
        const pswapCreateRequest = client.newPswapCreateTransactionRequest(
          accountAId,
          faucetAId,
          BigInt(100),
          faucetBId,
          BigInt(50),
          window.NoteType.Private
        );

        const expectedOutputNotes = pswapCreateRequest.expectedOutputOwnNotes();

        const createResult =
          await window.helpers.executeAndApplyTransaction(
            accountAId,
            pswapCreateRequest,
            undefined
          );

        await window.helpers.waitForTransaction(
          createResult.executedTransaction().id().toHex()
        );

        // Retrieve the PSWAP note
        const pswapNoteId = expectedOutputNotes[0].id().toString();
        const inputNoteRecord = await client.getInputNote(pswapNoteId);
        if (!inputNoteRecord) {
          throw new Error(`PSWAP note with ID ${pswapNoteId} not found`);
        }
        const pswapNote = inputNoteRecord.toNote();

        // Account B consumes the PSWAP note with full fill (50 of asset B)
        const pswapConsumeRequest = client.newPswapConsumeTransactionRequest(
          pswapNote,
          accountBId,
          BigInt(50),
          BigInt(0)
        );

        const expectedPaybackNoteDetails = pswapConsumeRequest
          .expectedFutureNotes()
          .map((futureNote) => futureNote.noteDetails);

        const consumeResult =
          await window.helpers.executeAndApplyTransaction(
            accountBId,
            pswapConsumeRequest,
            undefined
          );

        await window.helpers.waitForTransaction(
          consumeResult.executedTransaction().id().toHex()
        );

        // Account A consumes the payback (P2ID) note
        const paybackNoteId = expectedPaybackNoteDetails[0].id().toString();
        const paybackNoteRecord = await client.getInputNote(paybackNoteId);
        if (!paybackNoteRecord) {
          throw new Error(
            `Payback note with ID ${paybackNoteId} not found`
          );
        }
        const paybackNote = paybackNoteRecord.toNote();
        const consumePaybackRequest = client.newConsumeTransactionRequest([
          paybackNote,
        ]);

        const paybackConsumeResult =
          await window.helpers.executeAndApplyTransaction(
            accountAId,
            consumePaybackRequest,
            undefined
          );

        await window.helpers.waitForTransaction(
          paybackConsumeResult.executedTransaction().id().toHex()
        );

        // Fetch assets from both accounts
        const accountA = await client.getAccount(accountAId);
        const accountAAssets = accountA
          ?.vault()
          .fungibleAssets()
          .map((asset) => ({
            assetId: asset.faucetId().toString(),
            amount: asset.amount().toString(),
          }));

        const accountB = await client.getAccount(accountBId);
        const accountBAssets = accountB
          ?.vault()
          .fungibleAssets()
          .map((asset) => ({
            assetId: asset.faucetId().toString(),
            amount: asset.amount().toString(),
          }));

        return { accountAAssets, accountBAssets };
      },
      {
        _accountAId: accountA,
        _accountBId: accountB,
        _faucetAId: faucetA,
        _faucetBId: faucetB,
      }
    );

    // Account A: started with 1000 of A, offered 100, got 50 of B
    const aA = result.accountAAssets!.find((a) => a.assetId === faucetA);
    expect(aA, `Expected to find asset ${faucetA} on Account A`).toBeTruthy();
    expect(BigInt(aA!.amount)).toEqual(900n);

    const aB = result.accountAAssets!.find((a) => a.assetId === faucetB);
    expect(aB, `Expected to find asset ${faucetB} on Account A`).toBeTruthy();
    expect(BigInt(aB!.amount)).toEqual(50n);

    // Account B: started with 1000 of B, provided 50, got 100 of A
    const bA = result.accountBAssets!.find((a) => a.assetId === faucetA);
    expect(bA, `Expected to find asset ${faucetA} on Account B`).toBeTruthy();
    expect(BigInt(bA!.amount)).toEqual(100n);

    const bB = result.accountBAssets!.find((a) => a.assetId === faucetB);
    expect(bB, `Expected to find asset ${faucetB} on Account B`).toBeTruthy();
    expect(BigInt(bB!.amount)).toEqual(950n);
  });

  test("pswap cancel returns offered asset to creator", async ({ page }) => {
    test.setTimeout(480000);

    const { accountId: accountA, faucetId: faucetA } =
      await setupWalletAndFaucet(page);
    const { accountId: _accountB, faucetId: faucetB } =
      await setupWalletAndFaucet(page);

    await mintAndConsumeTransaction(page, accountA, faucetA, false);

    const result = await page.evaluate(
      async ({
        _accountAId,
        _faucetAId,
        _faucetBId,
      }: {
        _accountAId: string;
        _faucetAId: string;
        _faucetBId: string;
      }) => {
        const client = window.client;

        await client.syncState();

        const accountAId = window.AccountId.fromHex(_accountAId);
        const faucetAId = window.AccountId.fromHex(_faucetAId);
        const faucetBId = window.AccountId.fromHex(_faucetBId);

        // Create PSWAP note: account A offers 100 of asset A for 50 of asset B
        const pswapCreateRequest = client.newPswapCreateTransactionRequest(
          accountAId,
          faucetAId,
          BigInt(100),
          faucetBId,
          BigInt(50),
          window.NoteType.Private
        );

        const expectedOutputNotes = pswapCreateRequest.expectedOutputOwnNotes();

        const createResult =
          await window.helpers.executeAndApplyTransaction(
            accountAId,
            pswapCreateRequest,
            undefined
          );

        await window.helpers.waitForTransaction(
          createResult.executedTransaction().id().toHex()
        );

        // Retrieve the PSWAP note
        const pswapNoteId = expectedOutputNotes[0].id().toString();
        const inputNoteRecord = await client.getInputNote(pswapNoteId);
        if (!inputNoteRecord) {
          throw new Error(`PSWAP note with ID ${pswapNoteId} not found`);
        }
        const pswapNote = inputNoteRecord.toNote();

        // Cancel the PSWAP with account A
        const pswapCancelRequest =
          client.newPswapCancelTransactionRequest(pswapNote);

        const cancelResult =
          await window.helpers.executeAndApplyTransaction(
            accountAId,
            pswapCancelRequest,
            undefined
          );

        await window.helpers.waitForTransaction(
          cancelResult.executedTransaction().id().toHex()
        );

        // Fetch account A's assets after cancel
        const accountA = await client.getAccount(accountAId);
        const accountAAssets = accountA
          ?.vault()
          .fungibleAssets()
          .map((asset) => ({
            assetId: asset.faucetId().toString(),
            amount: asset.amount().toString(),
          }));

        return { accountAAssets };
      },
      {
        _accountAId: accountA,
        _faucetAId: faucetA,
        _faucetBId: faucetB,
      }
    );

    // Account A should have all 1000 of asset A back after cancel
    const aA = result.accountAAssets!.find((a) => a.assetId === faucetA);
    expect(aA, `Expected to find asset ${faucetA} on Account A`).toBeTruthy();
    expect(BigInt(aA!.amount)).toEqual(1000n);
  });
});
