// @ts-nocheck
import {
  test,
  expect,
  executeAndApplyTransaction,
  waitForTransaction,
} from "./test-setup";
import {
  createIntegrationClient,
  integrationMint,
  integrationConsume,
} from "./test-helpers";
import { getRpcUrl } from "./playwright.global.setup";

const badHexId =
  "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

async function setupIntegrationWalletAndFaucet(
  client: any,
  sdk: any
): Promise<{ walletId: any; faucetId: any }> {
  const wallet = await client.newWallet(
    sdk.AccountStorageMode.private(),
    true,
    sdk.AuthScheme.AuthRpoFalcon512
  );
  const faucet = await client.newFaucet(
    sdk.AccountStorageMode.private(),
    false,
    "DAG",
    8,
    sdk.u64(10000000),
    sdk.AuthScheme.AuthRpoFalcon512
  );
  return { walletId: wallet.id(), faucetId: faucet.id() };
}

async function setupMintedNote(
  client: any,
  sdk: any,
  opts?: { publicNote?: boolean }
): Promise<{ createdNoteId: string; walletId: any; faucetId: any }> {
  const { walletId, faucetId } = await setupIntegrationWalletAndFaucet(
    client,
    sdk
  );
  const { createdNoteId } = await integrationMint(
    client,
    sdk,
    walletId,
    faucetId,
    { publicNote: opts?.publicNote }
  );
  return { createdNoteId, walletId, faucetId };
}

async function setupConsumedNote(
  client: any,
  sdk: any,
  opts?: { publicNote?: boolean }
): Promise<{ consumedNoteId: string; walletId: any; faucetId: any }> {
  const { createdNoteId, walletId, faucetId } = await setupMintedNote(
    client,
    sdk,
    opts
  );
  await integrationConsume(client, sdk, walletId, faucetId, createdNoteId);
  return { consumedNoteId: createdNoteId, walletId, faucetId };
}

async function getConsumableNotes(
  client: any,
  sdk: any,
  accountId?: any
): Promise<
  {
    noteId: string;
    consumability: {
      accountId: string;
      consumableAfterBlock: number | undefined;
    }[];
  }[]
> {
  let records;
  if (accountId) {
    records = await client.getConsumableNotes(accountId);
  } else {
    records = await client.getConsumableNotes();
  }

  return records.map((record: any) => ({
    noteId: record.inputNoteRecord().id().toString(),
    consumability: record.noteConsumability().map((c: any) => ({
      accountId: c.accountId().toString(),
      consumableAfterBlock: c.consumptionStatus()?.consumableAfterBlock(),
    })),
  }));
}

test.describe("get_input_note", () => {
  test("retrieve input note that does not exist", async ({ sdk }) => {
    const result = await createIntegrationClient();
    test.skip(!result, "requires running node");
    const { client } = result;

    await setupIntegrationWalletAndFaucet(client, sdk);
    const note = await client.getInputNote(badHexId);
    expect(note).toBeUndefined();
  });

  test("retrieve an input note that does exist", async ({ sdk }) => {
    const result = await createIntegrationClient();
    test.skip(!result, "requires running node");
    const { client } = result;

    const { consumedNoteId } = await setupConsumedNote(client, sdk);

    // Test both the existing client method and new RpcClient
    const note = await client.getInputNote(consumedNoteId);
    expect(note).toBeTruthy();
    expect(note.id().toString()).toEqual(consumedNoteId);

    // Test RpcClient.getNotesById
    const endpoint = new sdk.Endpoint(getRpcUrl());
    const rpcClient = new sdk.RpcClient(endpoint);

    const noteId = sdk.NoteId.fromHex(consumedNoteId);
    const fetchedNotes = await rpcClient.getNotesById([noteId]);

    const rpcResult = fetchedNotes.map((fetchedNote: any) => ({
      noteId: fetchedNote.noteId.toString(),
      hasMetadata: !!fetchedNote.metadata,
      noteType: fetchedNote.noteType,
      hasNote: !!fetchedNote.note,
    }));

    // Assert on FetchedNote properties
    expect(rpcResult).toHaveLength(1);
    expect(rpcResult[0].noteId).toEqual(consumedNoteId);
    expect(rpcResult[0].hasMetadata).toBe(true);
    expect(rpcResult[0].hasNote).toBe(false); // Private notes don't include note
  });

  test("get note script by root", async ({ sdk }) => {
    const result = await createIntegrationClient();
    test.skip(!result, "requires running node");
    const { client } = result;

    await setupIntegrationWalletAndFaucet(client, sdk);

    // First, we need to get a note script root from an existing note
    const { consumedNoteId } = await setupConsumedNote(client, sdk, {
      publicNote: true,
    });

    // Get the note to extract its script root
    const endpoint = new sdk.Endpoint(getRpcUrl());
    const rpcClient = new sdk.RpcClient(endpoint);

    const noteIdObj = sdk.NoteId.fromHex(consumedNoteId);
    const fetchedNotes = await rpcClient.getNotesById([noteIdObj]);

    let scriptRootHex = "";
    let hasScript = false;

    if (fetchedNotes.length > 0 && fetchedNotes[0].note) {
      const scriptRoot = fetchedNotes[0].note.script().root();
      scriptRootHex = scriptRoot.toHex();
      hasScript = true;
    }

    // Test GetNoteScriptByRoot endpoint
    const scriptRoot = sdk.Word.fromHex(scriptRootHex);
    const noteScript = await rpcClient.getNoteScriptByRoot(scriptRoot);

    expect(!!noteScript).toBe(true);
    expect(noteScript ? noteScript.root().toHex() : null).toEqual(
      scriptRootHex
    );
  });

  test("sync notes by tag and check nullifier commit height", async ({
    sdk,
  }) => {
    const result = await createIntegrationClient();
    test.skip(!result, "requires running node");
    const { client } = result;

    const { consumedNoteId } = await setupConsumedNote(client, sdk, {
      publicNote: true,
    });

    const endpoint = new sdk.Endpoint(getRpcUrl());
    const rpcClient = new sdk.RpcClient(endpoint);

    const noteIdObj = sdk.NoteId.fromHex(consumedNoteId);
    const fetchedNotes = await rpcClient.getNotesById([noteIdObj]);

    if (fetchedNotes.length === 0) {
      expect(false).toBe(true); // Should not happen
      return;
    }

    const note = fetchedNotes[0].note;
    const tag = fetchedNotes[0].metadata.tag();

    const syncInfo = await rpcClient.syncNotes(0, undefined, [tag]);
    const syncedNoteIds = syncInfo
      .notes()
      .map((synced: any) => synced.noteId().toString());

    const inputNote = await client.getInputNote(consumedNoteId);
    const nullifierWord = note
      ? note.nullifier()
      : inputNote
        ? sdk.Word.fromHex(inputNote.nullifier())
        : undefined;
    const commitHeight = nullifierWord
      ? await rpcClient.getNullifierCommitHeight(nullifierWord, 0)
      : undefined;

    expect(syncedNoteIds).toContain(consumedNoteId);
    expect(note ? note.nullifier().toHex() : undefined).toMatch(
      /^0x[0-9a-fA-F]+$/
    );
    expect(commitHeight).not.toBeUndefined();
  });
});

test.describe("get_input_notes", () => {
  test("note exists, note filter all", async ({ sdk }) => {
    const result = await createIntegrationClient();
    test.skip(!result, "requires running node");
    const { client } = result;

    const { consumedNoteId } = await setupConsumedNote(client, sdk);
    const filter = new sdk.NoteFilter(sdk.NoteFilterTypes.All);
    const notes = await client.getInputNotes(filter);
    const noteIds = notes.map((note: any) => note.id().toString());
    expect(noteIds.length).toBeGreaterThanOrEqual(1);
    expect(noteIds).toContain(consumedNoteId);
  });
});

test.describe("get_consumable_notes", () => {
  test("filter by account", async ({ sdk }) => {
    const result = await createIntegrationClient();
    test.skip(!result, "requires running node");
    const { client } = result;

    const { createdNoteId: noteId1, walletId: walletId1 } =
      await setupMintedNote(client, sdk);
    const accountId1 = walletId1;

    const consumableResult = await getConsumableNotes(client, sdk, accountId1);
    expect(consumableResult).toHaveLength(1);
    consumableResult.forEach((record: any) => {
      expect(record.consumability).toHaveLength(1);
      expect(record.consumability[0].accountId).toBe(accountId1.toString());
      expect(record.noteId).toBe(noteId1);
      expect(record.consumability[0].consumableAfterBlock).toBeUndefined();
    });
  });

  test("no filter by account", async ({ sdk }) => {
    const result = await createIntegrationClient();
    test.skip(!result, "requires running node");
    const { client } = result;

    const { createdNoteId: noteId1, walletId: walletId1 } =
      await setupMintedNote(client, sdk);
    const { createdNoteId: noteId2, walletId: walletId2 } =
      await setupMintedNote(client, sdk);
    const accountId1 = walletId1.toString();
    const accountId2 = walletId2.toString();

    const noteIds = new Set([noteId1, noteId2]);
    const accountIds = new Set([accountId1, accountId2]);
    const consumableResult = await getConsumableNotes(client, sdk);
    expect(noteIds).toEqual(
      new Set(consumableResult.map((r: any) => r.noteId))
    );
    expect(accountIds).toEqual(
      new Set(consumableResult.map((r: any) => r.consumability[0].accountId))
    );
    expect(consumableResult.length).toBeGreaterThanOrEqual(2);
    const consumableRecord1 = consumableResult.find(
      (r: any) => r.noteId === noteId1
    );
    const consumableRecord2 = consumableResult.find(
      (r: any) => r.noteId === noteId2
    );

    consumableRecord1!!.consumability.forEach((c: any) => {
      expect(c.accountId).toEqual(accountId1);
    });

    consumableRecord2!!.consumability.forEach((c: any) => {
      expect(c.accountId).toEqual(accountId2);
    });
  });

  test("p2ide consume after block", async ({ sdk }) => {
    const result = await createIntegrationClient();
    test.skip(!result, "requires running node");
    const { client } = result;

    const { walletId: senderWalletId, faucetId: senderFaucetId } =
      await setupIntegrationWalletAndFaucet(client, sdk);
    const { walletId: targetWalletId } = await setupIntegrationWalletAndFaucet(
      client,
      sdk
    );

    // Mint and consume to fund sender
    const { createdNoteId: mintedNoteId } = await integrationMint(
      client,
      sdk,
      senderWalletId,
      senderFaucetId
    );
    await integrationConsume(
      client,
      sdk,
      senderWalletId,
      senderFaucetId,
      mintedNoteId
    );

    // Get sync height
    const summary = await client.syncState();
    const recallHeight = summary.blockNum() + 30;

    // Send transaction
    const sendRequest = await client.newSendTransactionRequest(
      senderWalletId,
      targetWalletId,
      senderFaucetId,
      sdk.NoteType.Public,
      sdk.u64(100),
      recallHeight,
      null
    );
    const sendResult = await executeAndApplyTransaction(
      client,
      sdk,
      senderWalletId,
      sendRequest
    );
    await waitForTransaction(
      client,
      sdk,
      sendResult.executedTransaction().id().toHex()
    );

    const consumableRecipient = await getConsumableNotes(
      client,
      sdk,
      targetWalletId
    );
    const consumableSender = await getConsumableNotes(
      client,
      sdk,
      senderWalletId
    );
    expect(consumableSender.length).toBe(1);
    expect(consumableSender[0].consumability[0].consumableAfterBlock).toBe(
      recallHeight
    );
    expect(consumableRecipient.length).toBe(1);
    expect(
      consumableRecipient[0].consumability[0].consumableAfterBlock
    ).toBeUndefined();
  });
});

test.describe("createP2IDNote and createP2IDENote", () => {
  test("should create a proper consumable p2id note from the createP2IDNote function", async ({
    sdk,
  }) => {
    const result = await createIntegrationClient();
    test.skip(!result, "requires running node");
    const { client } = result;

    const { walletId: senderId, faucetId } =
      await setupIntegrationWalletAndFaucet(client, sdk);
    const { walletId: targetId } = await setupIntegrationWalletAndFaucet(
      client,
      sdk
    );

    // Mint and consume to fund sender
    const { createdNoteId: mintedNoteId } = await integrationMint(
      client,
      sdk,
      senderId,
      faucetId,
      { publicNote: true }
    );
    await integrationConsume(client, sdk, senderId, faucetId, mintedNoteId);

    let fungibleAsset = new sdk.FungibleAsset(faucetId, sdk.u64(10));
    let noteAssets = new sdk.NoteAssets([fungibleAsset]);
    let p2IdNote = sdk.Note.createP2IDNote(
      senderId,
      targetId,
      noteAssets,
      sdk.NoteType.Public,
      new sdk.NoteAttachment()
    );

    let outputNote = sdk.OutputNote.full(p2IdNote);

    let transactionRequest = new sdk.TransactionRequestBuilder()
      .withOwnOutputNotes([outputNote])
      .build();

    let transactionUpdate = await executeAndApplyTransaction(
      client,
      sdk,
      senderId,
      transactionRequest
    );

    await waitForTransaction(
      client,
      sdk,
      transactionUpdate.executedTransaction().id().toHex()
    );

    let createdNoteId = transactionUpdate
      .executedTransaction()
      .outputNotes()
      .notes()[0]
      .id()
      .toString();

    const inputNoteRecord = await client.getInputNote(createdNoteId);
    if (!inputNoteRecord) {
      throw new Error(`Note with ID ${createdNoteId} not found`);
    }

    const note = inputNoteRecord.toNote();
    let consumeTransactionRequest = client.newConsumeTransactionRequest([note]);

    let consumeTransactionUpdate = await executeAndApplyTransaction(
      client,
      sdk,
      targetId,
      consumeTransactionRequest
    );

    await waitForTransaction(
      client,
      sdk,
      consumeTransactionUpdate.executedTransaction().id().toHex()
    );

    let senderAccountBalance = (await client.getAccount(senderId))
      ?.vault()
      .getBalance(faucetId)
      .toString();
    let targetAccountBalance = (await client.getAccount(targetId))
      ?.vault()
      .getBalance(faucetId)
      .toString();

    expect(senderAccountBalance).toEqual("990");
    expect(targetAccountBalance).toEqual("10");
  });

  test("should create a proper consumable p2ide note from the createP2IDENote function", async ({
    sdk,
  }) => {
    test.slow();
    const result = await createIntegrationClient();
    test.skip(!result, "requires running node");
    const { client } = result;

    const { walletId: senderId, faucetId } =
      await setupIntegrationWalletAndFaucet(client, sdk);
    const { walletId: targetId } = await setupIntegrationWalletAndFaucet(
      client,
      sdk
    );

    // Mint and consume to fund sender
    const { createdNoteId: mintedNoteId } = await integrationMint(
      client,
      sdk,
      senderId,
      faucetId,
      { publicNote: true }
    );
    await integrationConsume(client, sdk, senderId, faucetId, mintedNoteId);

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

    let outputNote = sdk.OutputNote.full(p2IdeNote);

    let transactionRequest = new sdk.TransactionRequestBuilder()
      .withOwnOutputNotes([outputNote])
      .build();

    let transactionUpdate = await executeAndApplyTransaction(
      client,
      sdk,
      senderId,
      transactionRequest
    );

    await waitForTransaction(
      client,
      sdk,
      transactionUpdate.executedTransaction().id().toHex()
    );

    let createdNoteId = transactionUpdate
      .executedTransaction()
      .outputNotes()
      .notes()[0]
      .id()
      .toString();

    const inputNoteRecord = await client.getInputNote(createdNoteId);
    if (!inputNoteRecord) {
      throw new Error(`Note with ID ${createdNoteId} not found`);
    }

    const note = inputNoteRecord.toNote();
    let consumeTransactionRequest = client.newConsumeTransactionRequest([note]);

    let consumeTransactionUpdate = await executeAndApplyTransaction(
      client,
      sdk,
      targetId,
      consumeTransactionRequest
    );

    await waitForTransaction(
      client,
      sdk,
      consumeTransactionUpdate.executedTransaction().id().toHex()
    );

    let senderAccountBalance = (await client.getAccount(senderId))
      ?.vault()
      .getBalance(faucetId)
      .toString();
    let targetAccountBalance = (await client.getAccount(targetId))
      ?.vault()
      .getBalance(faucetId)
      .toString();

    expect(senderAccountBalance).toEqual("990");
    expect(targetAccountBalance).toEqual("10");
  });
});

// TODO:
test.describe("get_output_note", () => {});

test.describe("get_output_notes", () => {});
