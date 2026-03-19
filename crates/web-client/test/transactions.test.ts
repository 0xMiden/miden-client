// @ts-nocheck
import { test, expect } from "./test-setup";

// GET_TRANSACTIONS TESTS
// =======================================================================================================

test.describe("get_transactions tests", () => {
  test("get_transactions retrieves all transactions successfully", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk, helpers }) => {
      const { wallet, faucet } = await helpers.setupWalletAndFaucet();
      const { mintTransactionId, consumeTransactionId } =
        await helpers.mockMintAndConsume(wallet.id(), faucet.id());

      const transactions = await client.getTransactions(
        sdk.TransactionFilter.all()
      );
      const transactionIds = transactions.map((tx) => tx.id().toHex());
      const uncommitted = await client.getTransactions(
        sdk.TransactionFilter.uncommitted()
      );

      return {
        transactionIds,
        mintTransactionId,
        consumeTransactionId,
        uncommittedLength: uncommitted.length,
      };
    });

    expect(result.transactionIds).toContain(result.mintTransactionId);
    expect(result.transactionIds).toContain(result.consumeTransactionId);
    expect(result.uncommittedLength).toEqual(0);
  });

  test("get_transactions retrieves uncommitted transactions successfully", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk, helpers }) => {
      const { wallet, faucet } = await helpers.setupWalletAndFaucet();
      const { mintTransactionId, consumeTransactionId } =
        await helpers.mockMintAndConsume(wallet.id(), faucet.id());
      const { transactionId: uncommittedTransactionId } =
        await helpers.mockMint(wallet.id(), faucet.id(), { skipSync: true });

      const transactions = await client.getTransactions(
        sdk.TransactionFilter.all()
      );
      const transactionIds = transactions.map((tx) => tx.id().toHex());
      const uncommitted = await client.getTransactions(
        sdk.TransactionFilter.uncommitted()
      );
      const uncommittedTransactionIds = uncommitted.map((tx) =>
        tx.id().toHex()
      );

      return {
        transactionIds,
        mintTransactionId,
        consumeTransactionId,
        uncommittedTransactionId,
        uncommittedTransactionIds,
      };
    });

    expect(result.transactionIds).toContain(result.mintTransactionId);
    expect(result.transactionIds).toContain(result.consumeTransactionId);
    expect(result.transactionIds).toContain(result.uncommittedTransactionId);
    expect(result.transactionIds.length).toEqual(3);

    expect(result.uncommittedTransactionIds).toContain(
      result.uncommittedTransactionId
    );
    expect(result.uncommittedTransactionIds.length).toEqual(1);
  });

  test("get_transactions retrieves no transactions successfully", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk }) => {
      const transactions = await client.getTransactions(
        sdk.TransactionFilter.all()
      );
      const uncommitted = await client.getTransactions(
        sdk.TransactionFilter.uncommitted()
      );

      return {
        transactionsLength: transactions.length,
        uncommittedLength: uncommitted.length,
      };
    });

    expect(result.transactionsLength).toEqual(0);
    expect(result.uncommittedLength).toEqual(0);
  });

  test("get_transactions filters by specific transaction IDs successfully", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk, helpers }) => {
      const { wallet, faucet } = await helpers.setupWalletAndFaucet();
      await helpers.mockMintAndConsume(wallet.id(), faucet.id());

      const allTransactions = await client.getTransactions(
        sdk.TransactionFilter.all()
      );
      const firstTransactionId = allTransactions[0].id();
      const firstTxIdHex = firstTransactionId.toHex();

      const filter = sdk.TransactionFilter.ids([firstTransactionId]);
      const filteredTransactions = await client.getTransactions(filter);
      const filteredTransactionIds = filteredTransactions.map((tx) =>
        tx.id().toHex()
      );

      return {
        allLength: allTransactions.length,
        filteredLength: filteredTransactionIds.length,
        filteredTransactionIds,
        firstTxIdHex,
      };
    });

    expect(result.allLength).toEqual(2);
    expect(result.filteredLength).toEqual(1);
    expect(result.filteredTransactionIds).toContain(result.firstTxIdHex);
  });

  test("get_transactions filters expired transactions successfully", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk, helpers }) => {
      const { wallet, faucet } = await helpers.setupWalletAndFaucet();

      const { transactionId: committedTransactionId } = await helpers.mockMint(
        wallet.id(),
        faucet.id()
      );

      const { transactionId: uncommittedTransactionId } =
        await helpers.mockMint(wallet.id(), faucet.id(), { skipSync: true });

      const allTransactions = await client.getTransactions(
        sdk.TransactionFilter.all()
      );
      const allTransactionIds = allTransactions.map((tx) => tx.id().toHex());
      // Use the committed transaction's blockNum (the one with blockNum > 0)
      const committedTx = allTransactions.find((tx) => tx.blockNum() > 0);
      const currentBlockNum = committedTx.blockNum();

      const futureBlockNum = currentBlockNum + 10;
      const futureExpiredTransactions = await client.getTransactions(
        sdk.TransactionFilter.expiredBefore(futureBlockNum)
      );
      const futureExpiredTransactionIds = futureExpiredTransactions.map((tx) =>
        tx.id().toHex()
      );

      // Ensure pastBlockNum doesn't go negative — on mock chain, block numbers
      // start low (e.g. 1) and negative values overflow to large unsigned ints.
      const pastBlockNum = Math.max(0, currentBlockNum - 10);
      const pastExpiredTransactions = await client.getTransactions(
        sdk.TransactionFilter.expiredBefore(pastBlockNum)
      );
      const pastExpiredTransactionIds = pastExpiredTransactions.map((tx) =>
        tx.id().toHex()
      );

      return {
        futureExpiredTransactionIds,
        pastExpiredTransactionIds,
        allTransactionIds,
        committedTransactionId,
        uncommittedTransactionId,
      };
    });

    expect(result.futureExpiredTransactionIds.length).toEqual(1);
    expect(result.futureExpiredTransactionIds).toContain(
      result.uncommittedTransactionId
    );
    expect(result.pastExpiredTransactionIds.length).toEqual(0);
    expect(result.allTransactionIds.length).toEqual(2);
    expect(result.allTransactionIds).toContain(result.committedTransactionId);
    expect(result.allTransactionIds).toContain(result.uncommittedTransactionId);
  });
});

// COMPILE_TX_SCRIPT TESTS
// =======================================================================================================

test.describe("compile_tx_script tests", () => {
  test("compile_tx_script compiles script successfully", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
      await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );

      const builder = await client.createCodeBuilder();
      const compiledScript = builder.compileNoteScript(`
        begin
          push.0 push.0
          assert_eq
        end
      `);

      return { rootHexLength: compiledScript.root().toHex().length };
    });

    expect(result.rootHexLength).toBeGreaterThan(1);
  });

  test("compile_tx_script does not compile script successfully", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk }) => {
      await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );

      const builder = await client.createCodeBuilder();

      let errorMessage = null;
      try {
        builder.compileNoteScript("fakeScript");
      } catch (e) {
        errorMessage = String(e);
      }

      return { errorMessage };
    });

    expect(result.errorMessage).toMatch(/failed to compile note script:/);
  });
});
