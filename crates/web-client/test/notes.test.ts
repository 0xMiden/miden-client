// @ts-nocheck
import { test, expect } from "./test-setup";

test.describe("get_input_note", () => {
  test("retrieve input note that does not exist", async ({ run }) => {
    const result = await run(async ({ sdk, helpers }) => {
      const integration = await helpers.createIntegrationClient();
      if (!integration) return { skip: true };
      const { client: intClient } = integration;

      const badHexId =
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

      const wallet = await intClient.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const faucet = await intClient.newFaucet(
        sdk.AccountStorageMode.private(),
        false,
        "DAG",
        8,
        sdk.u64(10000000),
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const note = await intClient.getInputNote(badHexId);
      return { skip: false, noteExists: note !== undefined && note !== null };
    });
    if (result.skip) {
      test.skip(true, "requires running node");
      return;
    }
    expect(result.noteExists).toBe(false);
  });

  test("retrieve an input note that does exist", async ({ run }) => {
    test.slow();
    const result = await run(async ({ sdk, helpers }) => {
      const integration = await helpers.createIntegrationClient();
      if (!integration) return { skip: true };
      const { client: intClient } = integration;

      // Setup wallet and faucet
      const wallet = await intClient.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const faucet = await intClient.newFaucet(
        sdk.AccountStorageMode.private(),
        false,
        "DAG",
        8,
        sdk.u64(10000000),
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const walletId = wallet.id();
      const faucetId = faucet.id();

      // Mint
      await intClient.syncState();
      const mintRequest = await intClient.newMintTransactionRequest(
        walletId,
        faucetId,
        sdk.NoteType.Private,
        sdk.u64(1000)
      );
      const mintResult = await intClient.executeTransaction(
        faucetId,
        mintRequest
      );
      const prover = sdk.TransactionProver.newLocalProver();
      const mintProven = await intClient.proveTransaction(mintResult, prover);
      const mintHeight = await intClient.submitProvenTransaction(
        mintProven,
        mintResult
      );
      const mintUpdate = await intClient.applyTransaction(
        mintResult,
        mintHeight
      );
      const mintTxId = mintUpdate.executedTransaction().id().toHex();
      const createdNoteId = mintUpdate
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
        if (!ids.includes(mintTxId)) break;
        await new Promise((r) => setTimeout(r, 1000));
        timeWaited += 1000;
      }

      // Consume
      await intClient.syncState();
      const inputNoteRecord = await intClient.getInputNote(createdNoteId);
      const note = inputNoteRecord.toNote();
      const consumeRequest = intClient.newConsumeTransactionRequest([note]);
      const consumeResult = await intClient.executeTransaction(
        walletId,
        consumeRequest
      );
      const consumeProver = sdk.TransactionProver.newLocalProver();
      const consumeProven = await intClient.proveTransaction(
        consumeResult,
        consumeProver
      );
      const consumeHeight = await intClient.submitProvenTransaction(
        consumeProven,
        consumeResult
      );
      const consumeUpdate = await intClient.applyTransaction(
        consumeResult,
        consumeHeight
      );
      const consumeTxId = consumeUpdate.executedTransaction().id().toHex();

      // Wait for consume tx
      timeWaited = 0;
      while (timeWaited < 10000) {
        await intClient.syncState();
        const uncommitted = await intClient.getTransactions(
          sdk.TransactionFilter.uncommitted()
        );
        const ids = uncommitted.map((tx) => tx.id().toHex());
        if (!ids.includes(consumeTxId)) break;
        await new Promise((r) => setTimeout(r, 1000));
        timeWaited += 1000;
      }

      // Test getInputNote
      const retrievedNote = await intClient.getInputNote(createdNoteId);
      const noteExists = !!retrievedNote;
      const noteIdStr = retrievedNote ? retrievedNote.id().toString() : null;

      // Test RpcClient.getNotesById
      const rpcUrl = helpers.getRpcUrl();
      const endpoint = new sdk.Endpoint(rpcUrl);
      const rpcClient = new sdk.RpcClient(endpoint);

      const noteIdObj = sdk.NoteId.fromHex(createdNoteId);
      const fetchedNotes = await rpcClient.getNotesById([noteIdObj]);

      const rpcResult = fetchedNotes.map((fetchedNote) => ({
        noteId: fetchedNote.noteId.toString(),
        hasMetadata: !!fetchedNote.metadata,
        noteType: fetchedNote.noteType,
        hasNote: !!fetchedNote.note,
      }));

      return {
        skip: false,
        noteExists,
        noteIdStr,
        consumedNoteId: createdNoteId,
        rpcResult,
      };
    });
    if (result.skip) {
      test.skip(true, "requires running node");
      return;
    }
    expect(result.noteExists).toBe(true);
    expect(result.noteIdStr).toEqual(result.consumedNoteId);
    expect(result.rpcResult).toHaveLength(1);
    expect(result.rpcResult[0].noteId).toEqual(result.consumedNoteId);
    expect(result.rpcResult[0].hasMetadata).toBe(true);
    expect(result.rpcResult[0].hasNote).toBe(false); // Private notes don't include note
  });

  test("get note script by root", async ({ run }) => {
    test.slow();
    const result = await run(async ({ sdk, helpers }) => {
      const integration = await helpers.createIntegrationClient();
      if (!integration) return { skip: true };
      const { client: intClient } = integration;

      // Setup wallet and faucet
      const wallet = await intClient.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const faucet = await intClient.newFaucet(
        sdk.AccountStorageMode.private(),
        false,
        "DAG",
        8,
        sdk.u64(10000000),
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const walletId = wallet.id();
      const faucetId = faucet.id();

      // Mint (public note)
      await intClient.syncState();
      const mintRequest = await intClient.newMintTransactionRequest(
        walletId,
        faucetId,
        sdk.NoteType.Public,
        sdk.u64(1000)
      );
      const mintResult = await intClient.executeTransaction(
        faucetId,
        mintRequest
      );
      const prover = sdk.TransactionProver.newLocalProver();
      const mintProven = await intClient.proveTransaction(mintResult, prover);
      const mintHeight = await intClient.submitProvenTransaction(
        mintProven,
        mintResult
      );
      const mintUpdate = await intClient.applyTransaction(
        mintResult,
        mintHeight
      );
      const mintTxId = mintUpdate.executedTransaction().id().toHex();
      const createdNoteId = mintUpdate
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
        if (!ids.includes(mintTxId)) break;
        await new Promise((r) => setTimeout(r, 1000));
        timeWaited += 1000;
      }

      // Consume (public note)
      await intClient.syncState();
      const inputNoteRecord = await intClient.getInputNote(createdNoteId);
      const note = inputNoteRecord.toNote();
      const consumeRequest = intClient.newConsumeTransactionRequest([note]);
      const consumeResult = await intClient.executeTransaction(
        walletId,
        consumeRequest
      );
      const consumeProver = sdk.TransactionProver.newLocalProver();
      const consumeProven = await intClient.proveTransaction(
        consumeResult,
        consumeProver
      );
      const consumeHeight = await intClient.submitProvenTransaction(
        consumeProven,
        consumeResult
      );
      const consumeUpdate = await intClient.applyTransaction(
        consumeResult,
        consumeHeight
      );
      const consumeTxId = consumeUpdate.executedTransaction().id().toHex();

      // Wait for consume tx
      timeWaited = 0;
      while (timeWaited < 10000) {
        await intClient.syncState();
        const uncommitted = await intClient.getTransactions(
          sdk.TransactionFilter.uncommitted()
        );
        const ids = uncommitted.map((tx) => tx.id().toHex());
        if (!ids.includes(consumeTxId)) break;
        await new Promise((r) => setTimeout(r, 1000));
        timeWaited += 1000;
      }

      // Get the note via RPC to extract its script root
      const rpcUrl = helpers.getRpcUrl();
      const endpoint = new sdk.Endpoint(rpcUrl);
      const rpcClient = new sdk.RpcClient(endpoint);

      const noteIdObj = sdk.NoteId.fromHex(createdNoteId);
      const fetchedNotes = await rpcClient.getNotesById([noteIdObj]);

      let scriptRootHex = "";

      if (fetchedNotes.length > 0 && fetchedNotes[0].note) {
        const scriptRoot = fetchedNotes[0].note.script().root();
        scriptRootHex = scriptRoot.toHex();
      }

      // Test GetNoteScriptByRoot endpoint
      const scriptRoot = sdk.Word.fromHex(scriptRootHex);
      const noteScript = await rpcClient.getNoteScriptByRoot(scriptRoot);

      return {
        skip: false,
        hasNoteScript: !!noteScript,
        noteScriptRootHex: noteScript ? noteScript.root().toHex() : null,
        scriptRootHex,
      };
    });
    if (result.skip) {
      test.skip(true, "requires running node");
      return;
    }
    expect(result.hasNoteScript).toBe(true);
    expect(result.noteScriptRootHex).toEqual(result.scriptRootHex);
  });

  test("sync notes by tag and check nullifier commit height", async ({
    run,
  }) => {
    test.slow();
    const result = await run(async ({ sdk, helpers }) => {
      const integration = await helpers.createIntegrationClient();
      if (!integration) return { skip: true };
      const { client: intClient } = integration;

      // Setup wallet and faucet
      const wallet = await intClient.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const faucet = await intClient.newFaucet(
        sdk.AccountStorageMode.private(),
        false,
        "DAG",
        8,
        sdk.u64(10000000),
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const walletId = wallet.id();
      const faucetId = faucet.id();

      // Mint (public note)
      await intClient.syncState();
      const mintRequest = await intClient.newMintTransactionRequest(
        walletId,
        faucetId,
        sdk.NoteType.Public,
        sdk.u64(1000)
      );
      const mintResult = await intClient.executeTransaction(
        faucetId,
        mintRequest
      );
      const prover = sdk.TransactionProver.newLocalProver();
      const mintProven = await intClient.proveTransaction(mintResult, prover);
      const mintHeight = await intClient.submitProvenTransaction(
        mintProven,
        mintResult
      );
      const mintUpdate = await intClient.applyTransaction(
        mintResult,
        mintHeight
      );
      const mintTxId = mintUpdate.executedTransaction().id().toHex();
      const createdNoteId = mintUpdate
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
        if (!ids.includes(mintTxId)) break;
        await new Promise((r) => setTimeout(r, 1000));
        timeWaited += 1000;
      }

      // Consume (public note)
      await intClient.syncState();
      const inputNoteRecord = await intClient.getInputNote(createdNoteId);
      const note = inputNoteRecord.toNote();
      const consumeRequest = intClient.newConsumeTransactionRequest([note]);
      const consumeResult = await intClient.executeTransaction(
        walletId,
        consumeRequest
      );
      const consumeProver = sdk.TransactionProver.newLocalProver();
      const consumeProven = await intClient.proveTransaction(
        consumeResult,
        consumeProver
      );
      const consumeHeight = await intClient.submitProvenTransaction(
        consumeProven,
        consumeResult
      );
      const consumeUpdate = await intClient.applyTransaction(
        consumeResult,
        consumeHeight
      );
      const consumeTxId = consumeUpdate.executedTransaction().id().toHex();

      // Wait for consume tx
      timeWaited = 0;
      while (timeWaited < 10000) {
        await intClient.syncState();
        const uncommitted = await intClient.getTransactions(
          sdk.TransactionFilter.uncommitted()
        );
        const ids = uncommitted.map((tx) => tx.id().toHex());
        if (!ids.includes(consumeTxId)) break;
        await new Promise((r) => setTimeout(r, 1000));
        timeWaited += 1000;
      }

      const rpcUrl = helpers.getRpcUrl();
      const endpoint = new sdk.Endpoint(rpcUrl);
      const rpcClient = new sdk.RpcClient(endpoint);

      const noteIdObj = sdk.NoteId.fromHex(createdNoteId);
      const fetchedNotes = await rpcClient.getNotesById([noteIdObj]);

      if (fetchedNotes.length === 0) {
        return { skip: false, fetchedNotesEmpty: true };
      }

      const fetchedNote = fetchedNotes[0].note;
      const tag = fetchedNotes[0].metadata.tag();

      const syncInfo = await rpcClient.syncNotes(0, undefined, [tag]);
      const blocks = syncInfo.blocks();
      const syncedNotes = syncInfo.notes();
      const syncedNoteIds = syncedNotes.map((synced) =>
        synced.noteId().toString()
      );
      const syncedBlockNoteIds = blocks.flatMap((block) =>
        block.notes().map((synced) => synced.noteId().toString())
      );
      const firstSyncedNote = syncedNotes[0];
      const compatBlockHeader = syncInfo.blockHeader();
      const compatMmrPath = syncInfo.mmrPath();

      const retrievedInputNote = await intClient.getInputNote(createdNoteId);
      const nullifierWord = fetchedNote
        ? fetchedNote.nullifier()
        : retrievedInputNote
          ? sdk.Word.fromHex(retrievedInputNote.nullifier())
          : undefined;
      const commitHeight = nullifierWord
        ? await rpcClient.getNullifierCommitHeight(nullifierWord, 0)
        : undefined;

      const nullifierHex = fetchedNote
        ? fetchedNote.nullifier().toHex()
        : undefined;

      return {
        skip: false,
        fetchedNotesEmpty: false,
        syncedNoteIds,
        syncedBlockNoteIds,
        consumedNoteId: createdNoteId,
        chainTip: syncInfo.chainTip(),
        blockTo: syncInfo.blockTo(),
        compatBlockNum: compatBlockHeader?.blockNum(),
        firstBlockNum: blocks[0]?.blockHeader().blockNum(),
        compatMmrDepth: compatMmrPath?.depth(),
        firstBlockMmrDepth: blocks[0]?.mmrPath().depth(),
        firstSyncedNoteSender: firstSyncedNote?.sender().toString(),
        firstSyncedNoteMetadataSender: firstSyncedNote
          ?.metadata()
          .sender()
          .toString(),
        firstSyncedNoteIndex: firstSyncedNote?.noteIndex(),
        firstSyncedNoteProofIndex: firstSyncedNote
          ?.inclusionProof()
          .location()
          .blockNoteTreeIndex(),
        nullifierHex,
        nullifierWord: fetchedNote
          ? fetchedNote.nullifier().toHex()
          : undefined,
        commitHeightDefined: commitHeight !== undefined,
      };
    });
    if (result.skip) {
      test.skip(true, "requires running node");
      return;
    }
    if (result.fetchedNotesEmpty) {
      expect(false).toBe(true); // Should not happen
      return;
    }
    expect(result.syncedNoteIds).toContain(result.consumedNoteId);
    expect(result.syncedNoteIds).toEqual(result.syncedBlockNoteIds);
    expect(result.chainTip).toBeGreaterThanOrEqual(result.blockTo);
    expect(result.compatBlockNum).toEqual(result.firstBlockNum);
    expect(result.compatMmrDepth).toEqual(result.firstBlockMmrDepth);
    expect(result.firstSyncedNoteSender).toEqual(
      result.firstSyncedNoteMetadataSender
    );
    expect(result.firstSyncedNoteIndex).toEqual(
      result.firstSyncedNoteProofIndex
    );
    expect(result.nullifierHex).toMatch(/^0x[0-9a-fA-F]+$/);
    expect(result.nullifierWord).toEqual(result.nullifierHex);
    expect(result.commitHeightDefined).toBe(true);
  });
});

test.describe("get_input_notes", () => {
  test("note exists, note filter all", async ({ run }) => {
    test.slow();
    const result = await run(async ({ sdk, helpers }) => {
      const integration = await helpers.createIntegrationClient();
      if (!integration) return { skip: true };
      const { client: intClient } = integration;

      // Setup wallet and faucet
      const wallet = await intClient.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const faucet = await intClient.newFaucet(
        sdk.AccountStorageMode.private(),
        false,
        "DAG",
        8,
        sdk.u64(10000000),
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const walletId = wallet.id();
      const faucetId = faucet.id();

      // Mint
      await intClient.syncState();
      const mintRequest = await intClient.newMintTransactionRequest(
        walletId,
        faucetId,
        sdk.NoteType.Private,
        sdk.u64(1000)
      );
      const mintResult = await intClient.executeTransaction(
        faucetId,
        mintRequest
      );
      const prover = sdk.TransactionProver.newLocalProver();
      const mintProven = await intClient.proveTransaction(mintResult, prover);
      const mintHeight = await intClient.submitProvenTransaction(
        mintProven,
        mintResult
      );
      const mintUpdate = await intClient.applyTransaction(
        mintResult,
        mintHeight
      );
      const mintTxId = mintUpdate.executedTransaction().id().toHex();
      const createdNoteId = mintUpdate
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
        if (!ids.includes(mintTxId)) break;
        await new Promise((r) => setTimeout(r, 1000));
        timeWaited += 1000;
      }

      // Consume
      await intClient.syncState();
      const inputNoteRecord = await intClient.getInputNote(createdNoteId);
      const note = inputNoteRecord.toNote();
      const consumeRequest = intClient.newConsumeTransactionRequest([note]);
      const consumeResult = await intClient.executeTransaction(
        walletId,
        consumeRequest
      );
      const consumeProver = sdk.TransactionProver.newLocalProver();
      const consumeProven = await intClient.proveTransaction(
        consumeResult,
        consumeProver
      );
      const consumeHeight = await intClient.submitProvenTransaction(
        consumeProven,
        consumeResult
      );
      const consumeUpdate = await intClient.applyTransaction(
        consumeResult,
        consumeHeight
      );
      const consumeTxId = consumeUpdate.executedTransaction().id().toHex();

      // Wait for consume tx
      timeWaited = 0;
      while (timeWaited < 10000) {
        await intClient.syncState();
        const uncommitted = await intClient.getTransactions(
          sdk.TransactionFilter.uncommitted()
        );
        const ids = uncommitted.map((tx) => tx.id().toHex());
        if (!ids.includes(consumeTxId)) break;
        await new Promise((r) => setTimeout(r, 1000));
        timeWaited += 1000;
      }

      const filter = new sdk.NoteFilter(sdk.NoteFilterTypes.All);
      const notes = await intClient.getInputNotes(filter);
      const noteIds = notes.map((n) => n.id().toString());

      return {
        skip: false,
        noteIdsLength: noteIds.length,
        containsConsumedNote: noteIds.includes(createdNoteId),
        consumedNoteId: createdNoteId,
      };
    });
    if (result.skip) {
      test.skip(true, "requires running node");
      return;
    }
    expect(result.noteIdsLength).toBeGreaterThanOrEqual(1);
    expect(result.containsConsumedNote).toBe(true);
  });
});

test.describe("get_consumable_notes", () => {
  test("filter by account", async ({ run }) => {
    test.slow();
    const result = await run(async ({ sdk, helpers }) => {
      const integration = await helpers.createIntegrationClient();
      if (!integration) return { skip: true };
      const { client: intClient } = integration;

      // Setup wallet and faucet
      const wallet = await intClient.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const faucet = await intClient.newFaucet(
        sdk.AccountStorageMode.private(),
        false,
        "DAG",
        8,
        sdk.u64(10000000),
        sdk.AuthScheme.AuthRpoFalcon512
      );
      // Save IDs as strings early — WASM objects may be consumed by methods
      const walletIdStr = wallet.id().toString();
      const faucetIdStr = faucet.id().toString();
      const walletIdObj = wallet.id();
      const faucetIdObj = faucet.id();

      // Mint
      await intClient.syncState();
      const mintRequest = await intClient.newMintTransactionRequest(
        walletIdObj,
        faucetIdObj,
        sdk.NoteType.Private,
        sdk.u64(1000)
      );
      const mintResult = await intClient.executeTransaction(
        faucet.id(),
        mintRequest
      );
      const prover = sdk.TransactionProver.newLocalProver();
      const mintProven = await intClient.proveTransaction(mintResult, prover);
      const mintHeight = await intClient.submitProvenTransaction(
        mintProven,
        mintResult
      );
      const mintUpdate = await intClient.applyTransaction(
        mintResult,
        mintHeight
      );
      const mintTxId = mintUpdate.executedTransaction().id().toHex();
      const createdNoteId = mintUpdate
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
        if (!ids.includes(mintTxId)) break;
        await new Promise((r) => setTimeout(r, 1000));
        timeWaited += 1000;
      }

      const records = await intClient.getConsumableNotes(wallet.id());
      const consumableResult = [];
      for (const record of records) {
        const noteId = record.inputNoteRecord().id().toString();
        const consumability = [];
        for (const c of record.noteConsumability()) {
          const accId = c.accountId();
          const accIdStr = accId.toString();
          consumability.push({
            accountId: accIdStr,
            consumableAfterBlock: c.consumptionStatus()?.consumableAfterBlock(),
          });
        }
        consumableResult.push({ noteId, consumability });
      }

      return {
        skip: false,
        consumableResult,
        walletIdStr,
        createdNoteId,
      };
    });
    if (result.skip) {
      test.skip(true, "requires running node");
      return;
    }
    expect(result.consumableResult).toHaveLength(1);
    result.consumableResult.forEach((record) => {
      expect(record.consumability).toHaveLength(1);
      expect(record.consumability[0].accountId).toBe(result.walletIdStr);
      expect(record.noteId).toBe(result.createdNoteId);
      // napi returns null, browser returns undefined for Option::None
      expect(record.consumability[0].consumableAfterBlock).toBeFalsy();
    });
  });

  test("no filter by account", async ({ run }) => {
    test.slow();
    const result = await run(async ({ sdk, helpers }) => {
      const integration = await helpers.createIntegrationClient();
      if (!integration) return { skip: true };
      const { client: intClient } = integration;

      // Helper to mint a note for a fresh wallet
      async function mintNote() {
        const wallet = await intClient.newWallet(
          sdk.AccountStorageMode.private(),
          true,
          sdk.AuthScheme.AuthRpoFalcon512
        );
        const faucet = await intClient.newFaucet(
          sdk.AccountStorageMode.private(),
          false,
          "DAG",
          8,
          sdk.u64(10000000),
          sdk.AuthScheme.AuthRpoFalcon512
        );
        const walletId = wallet.id();
        const faucetId = faucet.id();

        await intClient.syncState();
        const mintRequest = await intClient.newMintTransactionRequest(
          walletId,
          faucetId,
          sdk.NoteType.Private,
          sdk.u64(1000)
        );
        const mintResult = await intClient.executeTransaction(
          faucetId,
          mintRequest
        );
        const prover = sdk.TransactionProver.newLocalProver();
        const mintProven = await intClient.proveTransaction(mintResult, prover);
        const mintHeight = await intClient.submitProvenTransaction(
          mintProven,
          mintResult
        );
        const mintUpdate = await intClient.applyTransaction(
          mintResult,
          mintHeight
        );
        const mintTxId = mintUpdate.executedTransaction().id().toHex();
        const createdNoteId = mintUpdate
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
          if (!ids.includes(mintTxId)) break;
          await new Promise((r) => setTimeout(r, 1000));
          timeWaited += 1000;
        }

        return {
          createdNoteId,
          walletIdStr: walletId.toString(),
        };
      }

      const mint1 = await mintNote();
      const mint2 = await mintNote();

      const records = await intClient.getConsumableNotes();
      const consumableResult = records.map((record) => ({
        noteId: record.inputNoteRecord().id().toString(),
        consumability: record.noteConsumability().map((c) => ({
          accountId: c.accountId().toString(),
          consumableAfterBlock: c.consumptionStatus()?.consumableAfterBlock(),
        })),
      }));

      return {
        skip: false,
        consumableResult,
        noteId1: mint1.createdNoteId,
        noteId2: mint2.createdNoteId,
        accountId1: mint1.walletIdStr,
        accountId2: mint2.walletIdStr,
      };
    });
    if (result.skip) {
      test.skip(true, "requires running node");
      return;
    }
    const noteIds = new Set([result.noteId1, result.noteId2]);
    const accountIds = new Set([result.accountId1, result.accountId2]);
    expect(noteIds).toEqual(
      new Set(result.consumableResult.map((r) => r.noteId))
    );
    expect(accountIds).toEqual(
      new Set(result.consumableResult.map((r) => r.consumability[0].accountId))
    );
    expect(result.consumableResult.length).toBeGreaterThanOrEqual(2);
    const consumableRecord1 = result.consumableResult.find(
      (r) => r.noteId === result.noteId1
    );
    const consumableRecord2 = result.consumableResult.find(
      (r) => r.noteId === result.noteId2
    );

    consumableRecord1.consumability.forEach((c) => {
      expect(c.accountId).toEqual(result.accountId1);
    });

    consumableRecord2.consumability.forEach((c) => {
      expect(c.accountId).toEqual(result.accountId2);
    });
  });

  test("p2ide consume after block", async ({ run }) => {
    test.slow();
    const result = await run(async ({ sdk, helpers }) => {
      const integration = await helpers.createIntegrationClient();
      if (!integration) return { skip: true };
      const { client: intClient } = integration;

      // Setup sender wallet and faucet
      const senderWallet = await intClient.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const senderFaucet = await intClient.newFaucet(
        sdk.AccountStorageMode.private(),
        false,
        "DAG",
        8,
        sdk.u64(10000000),
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const senderWalletId = senderWallet.id();
      const senderFaucetId = senderFaucet.id();

      // Setup target wallet
      const targetWallet = await intClient.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const targetWalletId = targetWallet.id();

      // Mint
      await intClient.syncState();
      const mintRequest = await intClient.newMintTransactionRequest(
        senderWalletId,
        senderFaucetId,
        sdk.NoteType.Private,
        sdk.u64(1000)
      );
      let mintResult = await intClient.executeTransaction(
        senderFaucetId,
        mintRequest
      );
      let prover = sdk.TransactionProver.newLocalProver();
      let proven = await intClient.proveTransaction(mintResult, prover);
      let height = await intClient.submitProvenTransaction(proven, mintResult);
      const mintUpdate = await intClient.applyTransaction(mintResult, height);
      const mintTxId = mintUpdate.executedTransaction().id().toHex();
      const mintedNoteId = mintUpdate.createdNotes().notes()[0].id().toString();

      // Wait for mint tx
      let timeWaited = 0;
      while (timeWaited < 10000) {
        await intClient.syncState();
        const uncommitted = await intClient.getTransactions(
          sdk.TransactionFilter.uncommitted()
        );
        const ids = uncommitted.map((tx) => tx.id().toHex());
        if (!ids.includes(mintTxId)) break;
        await new Promise((r) => setTimeout(r, 1000));
        timeWaited += 1000;
      }

      // Consume
      await intClient.syncState();
      const inputNoteRecord = await intClient.getInputNote(mintedNoteId);
      const note = inputNoteRecord.toNote();
      const consumeRequest = intClient.newConsumeTransactionRequest([note]);
      let consumeResult = await intClient.executeTransaction(
        senderWalletId,
        consumeRequest
      );
      prover = sdk.TransactionProver.newLocalProver();
      proven = await intClient.proveTransaction(consumeResult, prover);
      height = await intClient.submitProvenTransaction(proven, consumeResult);
      const consumeUpdate = await intClient.applyTransaction(
        consumeResult,
        height
      );
      const consumeTxId = consumeUpdate.executedTransaction().id().toHex();

      // Wait for consume tx
      timeWaited = 0;
      while (timeWaited < 10000) {
        await intClient.syncState();
        const uncommitted = await intClient.getTransactions(
          sdk.TransactionFilter.uncommitted()
        );
        const ids = uncommitted.map((tx) => tx.id().toHex());
        if (!ids.includes(consumeTxId)) break;
        await new Promise((r) => setTimeout(r, 1000));
        timeWaited += 1000;
      }

      // Get sync height
      const summary = await intClient.syncState();
      const recallHeight = summary.blockNum() + 30;

      // Send transaction
      const sendRequest = await intClient.newSendTransactionRequest(
        senderWalletId,
        targetWalletId,
        senderFaucetId,
        sdk.NoteType.Public,
        sdk.u64(100),
        recallHeight,
        null
      );
      let sendResult = await intClient.executeTransaction(
        senderWalletId,
        sendRequest
      );
      prover = sdk.TransactionProver.newLocalProver();
      proven = await intClient.proveTransaction(sendResult, prover);
      height = await intClient.submitProvenTransaction(proven, sendResult);
      const sendUpdate = await intClient.applyTransaction(sendResult, height);
      const sendTxId = sendUpdate.executedTransaction().id().toHex();

      // Wait for send tx
      timeWaited = 0;
      while (timeWaited < 10000) {
        await intClient.syncState();
        const uncommitted = await intClient.getTransactions(
          sdk.TransactionFilter.uncommitted()
        );
        const ids = uncommitted.map((tx) => tx.id().toHex());
        if (!ids.includes(sendTxId)) break;
        await new Promise((r) => setTimeout(r, 1000));
        timeWaited += 1000;
      }

      // Get consumable notes
      const recipientRecords =
        await intClient.getConsumableNotes(targetWalletId);
      const consumableRecipient = recipientRecords.map((record) => ({
        noteId: record.inputNoteRecord().id().toString(),
        consumability: record.noteConsumability().map((c) => ({
          accountId: c.accountId().toString(),
          consumableAfterBlock: c.consumptionStatus()?.consumableAfterBlock(),
        })),
      }));

      const senderRecords = await intClient.getConsumableNotes(senderWalletId);
      const consumableSender = senderRecords.map((record) => ({
        noteId: record.inputNoteRecord().id().toString(),
        consumability: record.noteConsumability().map((c) => ({
          accountId: c.accountId().toString(),
          consumableAfterBlock: c.consumptionStatus()?.consumableAfterBlock(),
        })),
      }));

      return {
        skip: false,
        consumableSender,
        consumableRecipient,
        recallHeight,
      };
    });
    if (result.skip) {
      test.skip(true, "requires running node");
      return;
    }
    expect(result.consumableSender.length).toBe(1);
    expect(
      result.consumableSender[0].consumability[0].consumableAfterBlock
    ).toBe(result.recallHeight);
    expect(result.consumableRecipient.length).toBe(1);
    // napi returns null, browser returns undefined for Option::None
    expect(
      result.consumableRecipient[0].consumability[0].consumableAfterBlock
    ).toBeFalsy();
  });
});

test.describe("createP2IDNote and createP2IDENote", () => {
  test("should create a proper consumable p2id note from the createP2IDNote function", async ({
    run,
  }) => {
    test.slow();
    const result = await run(async ({ sdk, helpers }) => {
      const integration = await helpers.createIntegrationClient();
      if (!integration) return { skip: true };
      const { client: intClient } = integration;

      // Setup sender
      const sender = await intClient.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const faucet = await intClient.newFaucet(
        sdk.AccountStorageMode.private(),
        false,
        "DAG",
        8,
        sdk.u64(10000000),
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const senderId = sender.id();
      const faucetId = faucet.id();

      // Setup target
      const target = await intClient.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const targetId = target.id();

      // Mint (public note) and wait
      await intClient.syncState();
      const mintRequest = await intClient.newMintTransactionRequest(
        senderId,
        faucetId,
        sdk.NoteType.Public,
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
      const mintedNoteId = execUpdate.createdNotes().notes()[0].id().toString();

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

      // Consume minted note to fund sender
      await intClient.syncState();
      let inputNoteRecord = await intClient.getInputNote(mintedNoteId);
      let note = inputNoteRecord.toNote();
      let consumeRequest = intClient.newConsumeTransactionRequest([note]);
      execResult = await intClient.executeTransaction(senderId, consumeRequest);
      prover = sdk.TransactionProver.newLocalProver();
      proven = await intClient.proveTransaction(execResult, prover);
      height = await intClient.submitProvenTransaction(proven, execResult);
      execUpdate = await intClient.applyTransaction(execResult, height);
      txId = execUpdate.executedTransaction().id().toHex();

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

      // Create P2ID note
      let fungibleAsset = new sdk.FungibleAsset(faucetId, sdk.u64(10));
      let noteAssets = new sdk.NoteAssets([fungibleAsset]);
      let p2IdNote = sdk.Note.createP2IDNote(
        senderId,
        targetId,
        noteAssets,
        sdk.NoteType.Public,
        new sdk.NoteAttachment()
      );

      let ownNotes;
      if (sdk.NoteArray) {
        ownNotes = new sdk.NoteArray();
        ownNotes.push(p2IdNote);
      } else {
        ownNotes = [p2IdNote];
      }
      let transactionRequest = new sdk.TransactionRequestBuilder()
        .withOwnOutputNotes(ownNotes)
        .build();

      execResult = await intClient.executeTransaction(
        senderId,
        transactionRequest
      );
      prover = sdk.TransactionProver.newLocalProver();
      proven = await intClient.proveTransaction(execResult, prover);
      height = await intClient.submitProvenTransaction(proven, execResult);
      execUpdate = await intClient.applyTransaction(execResult, height);
      txId = execUpdate.executedTransaction().id().toHex();

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

      let createdNoteId = execUpdate
        .executedTransaction()
        .outputNotes()
        .notes()[0]
        .id()
        .toString();

      // Consume P2ID note by target
      inputNoteRecord = await intClient.getInputNote(createdNoteId);
      if (!inputNoteRecord) {
        throw new Error(`Note with ID ${createdNoteId} not found`);
      }

      note = inputNoteRecord.toNote();
      consumeRequest = intClient.newConsumeTransactionRequest([note]);

      execResult = await intClient.executeTransaction(targetId, consumeRequest);
      prover = sdk.TransactionProver.newLocalProver();
      proven = await intClient.proveTransaction(execResult, prover);
      height = await intClient.submitProvenTransaction(proven, execResult);
      execUpdate = await intClient.applyTransaction(execResult, height);
      txId = execUpdate.executedTransaction().id().toHex();

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

      let senderAccountBalance = (await intClient.getAccount(senderId))
        ?.vault()
        .getBalance(faucetId)
        .toString();
      let targetAccountBalance = (await intClient.getAccount(targetId))
        ?.vault()
        .getBalance(faucetId)
        .toString();

      return {
        skip: false,
        senderAccountBalance,
        targetAccountBalance,
      };
    });
    if (result.skip) {
      test.skip(true, "requires running node");
      return;
    }
    expect(result.senderAccountBalance).toEqual("990");
    expect(result.targetAccountBalance).toEqual("10");
  });

  test("should create a proper consumable p2ide note from the createP2IDENote function", async ({
    run,
  }) => {
    test.slow();
    const result = await run(async ({ sdk, helpers }) => {
      const integration = await helpers.createIntegrationClient();
      if (!integration) return { skip: true };
      const { client: intClient } = integration;

      // Setup sender
      const sender = await intClient.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const faucet = await intClient.newFaucet(
        sdk.AccountStorageMode.private(),
        false,
        "DAG",
        8,
        sdk.u64(10000000),
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const senderId = sender.id();
      const faucetId = faucet.id();

      // Setup target
      const target = await intClient.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const targetId = target.id();

      // Mint (public note) and wait
      await intClient.syncState();
      const mintRequest = await intClient.newMintTransactionRequest(
        senderId,
        faucetId,
        sdk.NoteType.Public,
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
      const mintedNoteId = execUpdate.createdNotes().notes()[0].id().toString();

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

      // Consume minted note to fund sender
      await intClient.syncState();
      let inputNoteRecord = await intClient.getInputNote(mintedNoteId);
      let note = inputNoteRecord.toNote();
      let consumeRequest = intClient.newConsumeTransactionRequest([note]);
      execResult = await intClient.executeTransaction(senderId, consumeRequest);
      prover = sdk.TransactionProver.newLocalProver();
      proven = await intClient.proveTransaction(execResult, prover);
      height = await intClient.submitProvenTransaction(proven, execResult);
      execUpdate = await intClient.applyTransaction(execResult, height);
      txId = execUpdate.executedTransaction().id().toHex();

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

      // Create P2IDE note
      let fungibleAsset = new sdk.FungibleAsset(faucetId, sdk.u64(10));
      let noteAssets = new sdk.NoteAssets([fungibleAsset]);
      let p2IdeNote = sdk.Note.createP2IDENote(
        senderId,
        targetId,
        noteAssets,
        null,
        null,
        sdk.NoteType.Public,
        new sdk.NoteAttachment()
      );

      let ownNotes;
      if (sdk.NoteArray) {
        ownNotes = new sdk.NoteArray();
        ownNotes.push(p2IdeNote);
      } else {
        ownNotes = [p2IdeNote];
      }
      let transactionRequest = new sdk.TransactionRequestBuilder()
        .withOwnOutputNotes(ownNotes)
        .build();

      execResult = await intClient.executeTransaction(
        senderId,
        transactionRequest
      );
      prover = sdk.TransactionProver.newLocalProver();
      proven = await intClient.proveTransaction(execResult, prover);
      height = await intClient.submitProvenTransaction(proven, execResult);
      execUpdate = await intClient.applyTransaction(execResult, height);
      txId = execUpdate.executedTransaction().id().toHex();

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

      let createdNoteId = execUpdate
        .executedTransaction()
        .outputNotes()
        .notes()[0]
        .id()
        .toString();

      // Consume P2IDE note by target
      inputNoteRecord = await intClient.getInputNote(createdNoteId);
      if (!inputNoteRecord) {
        throw new Error(`Note with ID ${createdNoteId} not found`);
      }

      note = inputNoteRecord.toNote();
      consumeRequest = intClient.newConsumeTransactionRequest([note]);

      execResult = await intClient.executeTransaction(targetId, consumeRequest);
      prover = sdk.TransactionProver.newLocalProver();
      proven = await intClient.proveTransaction(execResult, prover);
      height = await intClient.submitProvenTransaction(proven, execResult);
      execUpdate = await intClient.applyTransaction(execResult, height);
      txId = execUpdate.executedTransaction().id().toHex();

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

      let senderAccountBalance = (await intClient.getAccount(senderId))
        ?.vault()
        .getBalance(faucetId)
        .toString();
      let targetAccountBalance = (await intClient.getAccount(targetId))
        ?.vault()
        .getBalance(faucetId)
        .toString();

      return {
        skip: false,
        senderAccountBalance,
        targetAccountBalance,
      };
    });
    if (result.skip) {
      test.skip(true, "requires running node");
      return;
    }
    expect(result.senderAccountBalance).toEqual("990");
    expect(result.targetAccountBalance).toEqual("10");
  });
});

// TODO:
test.describe("get_output_note", () => {});

test.describe("get_output_notes", () => {});
