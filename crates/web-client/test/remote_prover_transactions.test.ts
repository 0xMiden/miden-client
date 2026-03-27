// @ts-nocheck
import test from "./playwright.global.setup";
import { expect } from "@playwright/test";
import {
  setupWalletAndFaucet,
  mintTransaction,
  consumeTransaction,
  sendTransaction,
} from "./webClientTestUtils";

// Remote prover transaction tests.
// These re-run key transaction flows using a remote prover.
// They require a running node + remote prover service (REMOTE_PROVER env).
// The CI grep filter matches test names containing "with remote prover".

test.describe("remote prover transaction tests", () => {
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
});
