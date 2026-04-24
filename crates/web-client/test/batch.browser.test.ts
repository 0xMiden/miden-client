import test from "./playwright.global.setup";
import { expect, Page } from "@playwright/test";
import {
  setupWalletAndFaucet,
  mintAndConsumeTransaction,
} from "./webClientTestUtils";

// SUBMIT_NEW_TRANSACTION_BATCH TESTS
// ================================================================================================

interface BatchSubmitResult {
  blockNum: number;
  nonceBefore: string;
  nonceAfter: string;
}

const submitTwoTxBatch = async (
  testingPage: Page,
  senderAccount: string,
  targetAccount: string,
  faucetAccount: string
): Promise<BatchSubmitResult> => {
  return await testingPage.evaluate(
    async ({ _senderAccount, _targetAccount, _faucetAccount }) => {
      const client = window.client;

      await client.syncState();

      const senderAccountId = window.AccountId.fromHex(_senderAccount);
      const targetAccountId = window.AccountId.fromHex(_targetAccount);
      const faucetAccountId = window.AccountId.fromHex(_faucetAccount);

      // Snapshot the sender's nonce before the batch. The sender's wallet uses
      // IncrNonce auth, so nonce advances by exactly 1 per tx — checking a 2-tx
      // batch advances nonce by exactly 2 is a direct test of BatchBuilder's
      // per-push account state stacking.
      const senderBefore = await client.getAccount(senderAccountId);
      const nonceBefore = senderBefore!.nonce()!.toString();

      // Build two P2ID send requests of 50 tokens each.
      const sendRequest1 = await client.newSendTransactionRequest(
        senderAccountId,
        targetAccountId,
        faucetAccountId,
        window.NoteType.Public,
        BigInt(50),
        null,
        null
      );
      const sendRequest2 = await client.newSendTransactionRequest(
        senderAccountId,
        targetAccountId,
        faucetAccountId,
        window.NoteType.Public,
        BigInt(50),
        null,
        null
      );

      const blockNum = await client.submitNewTransactionBatch(senderAccountId, [
        sendRequest1.serialize(),
        sendRequest2.serialize(),
      ]);

      // Poll until the sender nonce has advanced by 2, giving the node time to
      // finalize the batch's block.
      const targetNonce = BigInt(nonceBefore) + BigInt(2);
      let nonceAfter = nonceBefore;
      for (let attempt = 0; attempt < 60; attempt++) {
        await client.syncState();
        const senderAfter = await client.getAccount(senderAccountId);
        nonceAfter = senderAfter!.nonce()!.toString();
        if (BigInt(nonceAfter) >= targetNonce) {
          break;
        }
        await new Promise((resolve) => setTimeout(resolve, 2000));
      }

      return { blockNum, nonceBefore, nonceAfter };
    },
    {
      _senderAccount: senderAccount,
      _targetAccount: targetAccount,
      _faucetAccount: faucetAccount,
    }
  );
};

test.describe("submitNewTransactionBatch tests", () => {
  test("2-tx batch advances sender nonce by exactly 2", async ({ page }) => {
    test.setTimeout(900000);

    // Set up a sender wallet with funds and a separate target wallet.
    const { accountId: senderAccountId, faucetId } =
      await setupWalletAndFaucet(page);
    const { accountId: targetAccountId } = await setupWalletAndFaucet(page);

    // Fund the sender with tokens from the faucet.
    await mintAndConsumeTransaction(page, senderAccountId, faucetId);

    const result = await submitTwoTxBatch(
      page,
      senderAccountId,
      targetAccountId,
      faucetId
    );

    expect(result.blockNum).toBeGreaterThan(0);

    // Explicit state-stacking check: if BatchBuilder didn't stack state between
    // pushes, both txs would carry the same initial_account_state and the node
    // would reject the batch — the delta below would be 0 or 1, not 2.
    const delta = BigInt(result.nonceAfter) - BigInt(result.nonceBefore);
    expect(delta).toEqual(BigInt(2));
  });
});
