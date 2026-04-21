// @ts-nocheck
import { test, expect } from "./test-setup";

test.describe("import from seed", () => {
  test("should import same public account from seed", async ({ run }) => {
    test.slow();
    const result = await run(async ({ sdk, helpers }) => {
      const integration = await helpers.createIntegrationClient();
      if (!integration) return { skip: true };
      const { client: intClient } = integration;

      const walletSeed = new Uint8Array(32);
      crypto.getRandomValues(walletSeed);

      const mutable = false;

      const initialWallet = await intClient.newWallet(
        sdk.AccountStorageMode.public(),
        mutable,
        sdk.AuthScheme.AuthRpoFalcon512,
        walletSeed
      );
      const initialWalletId = initialWallet.id();

      const faucet = await intClient.newFaucet(
        sdk.AccountStorageMode.public(),
        false,
        "DAG",
        8,
        sdk.u64(10000000),
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const faucetId = faucet.id();

      // Mint
      await intClient.syncState();
      const mintRequest = await intClient.newMintTransactionRequest(
        initialWalletId,
        faucetId,
        sdk.NoteType.Private,
        sdk.u64(1000)
      );
      let execResult = await intClient.executeTransaction(
        faucetId,
        mintRequest
      );
      let prover = sdk.TransactionProver.newLocalProver();
      let proven = await intClient.proveTransaction(execResult, prover);
      let height = await intClient.submitProvenTransaction(proven, execResult);
      let execUpdate = await intClient.applyTransaction(execResult, height);
      let txId = execUpdate.executedTransaction().id().toHex();
      const createdNoteId = execUpdate
        .createdNotes()
        .notes()[0]
        .id()
        .toString();

      // Wait for mint tx
      let timeWaited = 0;
      while (timeWaited < 10000) {
        await intClient.syncState();
        const uncommitted = await intClient.getTransactions(
          sdk.TransactionFilter.uncommitted()
        );
        const ids = uncommitted.map((tx) => tx.id().toHex());
        if (!ids.includes(txId)) break;
        await new Promise((r) => setTimeout(r, 1000));
        timeWaited += 1000;
      }

      // Consume
      await intClient.syncState();
      const inputNoteRecord = await intClient.getInputNote(createdNoteId);
      const note = inputNoteRecord.toNote();
      const consumeRequest = intClient.newConsumeTransactionRequest([note]);
      execResult = await intClient.executeTransaction(
        initialWalletId,
        consumeRequest
      );
      prover = sdk.TransactionProver.newLocalProver();
      proven = await intClient.proveTransaction(execResult, prover);
      height = await intClient.submitProvenTransaction(proven, execResult);
      execUpdate = await intClient.applyTransaction(execResult, height);
      txId = execUpdate.executedTransaction().id().toHex();

      // Wait for consume tx
      timeWaited = 0;
      while (timeWaited < 10000) {
        await intClient.syncState();
        const uncommitted = await intClient.getTransactions(
          sdk.TransactionFilter.uncommitted()
        );
        const ids = uncommitted.map((tx) => tx.id().toHex());
        if (!ids.includes(txId)) break;
        await new Promise((r) => setTimeout(r, 1000));
        timeWaited += 1000;
      }

      const initialAccount = await intClient.getAccount(initialWalletId);
      const initialBalance = initialAccount
        .vault()
        .getBalance(faucetId)
        .toString();
      const initialCommitment = initialAccount.to_commitment().toHex();

      // Create a fresh client (separate store) and import the wallet from seed
      const integration2 = await helpers.createIntegrationClient();
      if (!integration2) return { skip: true };
      const { client: freshClient } = integration2;

      await freshClient.syncState();
      const restoredAccount = await freshClient.importPublicAccountFromSeed(
        walletSeed,
        mutable,
        sdk.AuthScheme.AuthRpoFalcon512
      );

      const restoredAccountId = restoredAccount.id().toString();

      const restoredAccountObj = await freshClient.getAccount(
        sdk.AccountId.fromHex(restoredAccountId)
      );
      const restoredAccountCommitment = restoredAccountObj
        .to_commitment()
        .toHex();

      const restoredBalance = restoredAccountObj
        .vault()
        .getBalance(sdk.AccountId.fromHex(faucetId.toString()));

      return {
        skip: false,
        initialWalletIdStr: initialWalletId.toString(),
        restoredAccountId,
        initialBalance,
        restoredBalance: restoredBalance.toString(),
        initialCommitment,
        restoredAccountCommitment,
      };
    });
    if (result.skip) {
      test.skip(true, "requires running node");
      return;
    }
    expect(result.restoredAccountId).toEqual(result.initialWalletIdStr);
    expect(result.restoredBalance).toEqual(result.initialBalance);
    expect(result.restoredAccountCommitment).toEqual(result.initialCommitment);
  });
});

test.describe("import public account by id", () => {
  test("should import public account from id", async ({ run }) => {
    test.slow();
    const result = await run(async ({ sdk, helpers }) => {
      const integration = await helpers.createIntegrationClient();
      if (!integration) return { skip: true };
      const { client: intClient } = integration;

      const walletSeed = new Uint8Array(32);
      crypto.getRandomValues(walletSeed);

      const mutable = false;

      const initialWallet = await intClient.newWallet(
        sdk.AccountStorageMode.public(),
        mutable,
        sdk.AuthScheme.AuthRpoFalcon512,
        walletSeed
      );
      const initialWalletId = initialWallet.id();

      const faucet = await intClient.newFaucet(
        sdk.AccountStorageMode.public(),
        false,
        "DAG",
        8,
        sdk.u64(10000000),
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const faucetId = faucet.id();

      // Mint
      await intClient.syncState();
      const mintRequest = await intClient.newMintTransactionRequest(
        initialWalletId,
        faucetId,
        sdk.NoteType.Private,
        sdk.u64(1000)
      );
      let execResult = await intClient.executeTransaction(
        faucetId,
        mintRequest
      );
      let prover = sdk.TransactionProver.newLocalProver();
      let proven = await intClient.proveTransaction(execResult, prover);
      let height = await intClient.submitProvenTransaction(proven, execResult);
      let execUpdate = await intClient.applyTransaction(execResult, height);
      let txId = execUpdate.executedTransaction().id().toHex();
      const createdNoteId = execUpdate
        .createdNotes()
        .notes()[0]
        .id()
        .toString();

      // Wait for mint tx
      let timeWaited = 0;
      while (timeWaited < 10000) {
        await intClient.syncState();
        const uncommitted = await intClient.getTransactions(
          sdk.TransactionFilter.uncommitted()
        );
        const ids = uncommitted.map((tx) => tx.id().toHex());
        if (!ids.includes(txId)) break;
        await new Promise((r) => setTimeout(r, 1000));
        timeWaited += 1000;
      }

      // Consume
      await intClient.syncState();
      const inputNoteRecord = await intClient.getInputNote(createdNoteId);
      const note = inputNoteRecord.toNote();
      const consumeRequest = intClient.newConsumeTransactionRequest([note]);
      execResult = await intClient.executeTransaction(
        initialWalletId,
        consumeRequest
      );
      prover = sdk.TransactionProver.newLocalProver();
      proven = await intClient.proveTransaction(execResult, prover);
      height = await intClient.submitProvenTransaction(proven, execResult);
      execUpdate = await intClient.applyTransaction(execResult, height);
      txId = execUpdate.executedTransaction().id().toHex();

      // Wait for consume tx
      timeWaited = 0;
      while (timeWaited < 10000) {
        await intClient.syncState();
        const uncommitted = await intClient.getTransactions(
          sdk.TransactionFilter.uncommitted()
        );
        const ids = uncommitted.map((tx) => tx.id().toHex());
        if (!ids.includes(txId)) break;
        await new Promise((r) => setTimeout(r, 1000));
        timeWaited += 1000;
      }

      const initialAccount = await intClient.getAccount(initialWalletId);
      const initialBalance = initialAccount
        .vault()
        .getBalance(faucetId)
        .toString();
      const initialCommitment = initialAccount.to_commitment().toHex();

      // Create a fresh client (separate store) and import by account ID
      const integration2 = await helpers.createIntegrationClient();
      if (!integration2) return { skip: true };
      const { client: freshClient } = integration2;

      const accountIdObj = sdk.AccountId.fromHex(initialWalletId.toString());
      await freshClient.importAccountById(accountIdObj);
      const restoredAccount = await freshClient.getAccount(accountIdObj);

      const restoredAccountId = restoredAccount.id().toString();
      const restoredAccountCommitment = restoredAccount.to_commitment().toHex();
      const restoredBalance = restoredAccount
        .vault()
        .getBalance(sdk.AccountId.fromHex(faucetId.toString()));

      return {
        skip: false,
        initialWalletIdStr: initialWalletId.toString(),
        restoredAccountId,
        initialBalance,
        restoredBalance: restoredBalance.toString(),
        initialCommitment,
        restoredAccountCommitment,
      };
    });
    if (result.skip) {
      test.skip(true, "requires running node");
      return;
    }
    expect(result.restoredAccountId).toEqual(result.initialWalletIdStr);
    expect(result.restoredBalance).toEqual(result.initialBalance);
    expect(result.restoredAccountCommitment).toEqual(result.initialCommitment);
  });
});
