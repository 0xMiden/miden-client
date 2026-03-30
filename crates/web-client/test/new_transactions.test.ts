// @ts-nocheck
import { test, expect } from "./test-setup";

// NEW_MINT_TRANSACTION TESTS
// =======================================================================================================

test.describe("mint transaction tests", () => {
  test("multiple mints and consumes complete successfully", async ({ run }) => {
    // This test was added in #995 to reproduce an issue in the web wallet.
    // It is useful because most tests consume the note right on the latest client block,
    // but this test mints 3 notes and consumes them after the fact. This ensures the
    // MMR data for old blocks is available and valid so that the notes can be consumed.
    const result = await run(async ({ client, sdk, helpers }) => {
      const { wallet, faucet } = await helpers.setupWalletAndFaucet();

      // Mint 3 notes
      const results = [];
      for (let i = 0; i < 3; i++) {
        results.push(await helpers.mockMint(wallet.id(), faucet.id()));
      }

      const mintCount = results.length;
      const allHaveOneOutput = results.every(
        (r) => r.numOutputNotesCreated === 1
      );

      // Consume all minted notes
      for (const { createdNoteId } of results) {
        await helpers.mockConsume(wallet.id(), createdNoteId);
      }

      const account = await client.getAccount(wallet.id());
      const balance = account.vault().getBalance(faucet.id()).toString();

      return { mintCount, allHaveOneOutput, balance };
    });

    expect(result.mintCount).toEqual(3);
    expect(result.allHaveOneOutput).toBe(true);
    expect(result.balance).toEqual("3000");
  });
});

// NEW_CONSUME_TRANSACTION TESTS
// =======================================================================================================

test.describe("consume transaction tests", () => {
  test("consume transaction completes successfully", async ({ run }) => {
    const result = await run(async ({ client, sdk, helpers }) => {
      const { wallet, faucet } = await helpers.setupWalletAndFaucet();
      const { targetAccountBalance } = await helpers.mockMintAndConsume(
        wallet.id(),
        faucet.id()
      );

      return { targetAccountBalance };
    });

    expect(result.targetAccountBalance).toEqual("1000");
  });
});

// NEW_SEND_TRANSACTION TESTS
// =======================================================================================================

test.describe("send transaction tests", () => {
  test("send transaction completes successfully", async ({ run }) => {
    const result = await run(async ({ client, sdk, helpers }) => {
      const { wallet: sender, faucet } = await helpers.setupWalletAndFaucet();
      const target = await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );

      const { sendCreatedNoteIds } = await helpers.mockSend(
        sender.id(),
        target.id(),
        faucet.id(),
        { recallHeight: 100 }
      );

      // Consume the sent note on the target account
      await helpers.mockConsume(target.id(), sendCreatedNoteIds[0]);

      const senderAccount = await client.getAccount(sender.id());
      const senderBalance = senderAccount
        .vault()
        .getBalance(faucet.id())
        .toString();

      const targetAccount = await client.getAccount(target.id());
      const targetBalance = targetAccount
        .vault()
        .getBalance(faucet.id())
        .toString();

      return { senderBalance, targetBalance };
    });

    expect(result.senderBalance).toEqual("900");
    expect(result.targetBalance).toEqual("100");
  });

  test("sends a public P2ID note, then receiver consumes it using the note's id", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk, helpers }) => {
      const { wallet: sender, faucet } = await helpers.setupWalletAndFaucet();
      const receiver = await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );

      // Fund sender
      await helpers.mockMintAndConsume(sender.id(), faucet.id());

      // Send public note
      const sendRequest = await client.newSendTransactionRequest(
        sender.id(),
        receiver.id(),
        faucet.id(),
        sdk.NoteType.Public,
        sdk.u64(100),
        null,
        null
      );
      const sendTxId = await client.submitNewTransaction(
        sender.id(),
        sendRequest
      );
      await client.proveBlock();
      await client.syncState();

      // Get the sent note ID
      const [sendTx] = await client.getTransactions(
        sdk.TransactionFilter.ids([sendTxId])
      );
      const sentNoteId = sendTx.outputNotes().notes()[0].id().toString();

      // Receiver consumes by note ID
      const inputNote = await client.getInputNote(sentNoteId);
      const note = inputNote.toNote();
      const consumeRequest = client.newConsumeTransactionRequest([note]);
      await client.submitNewTransaction(receiver.id(), consumeRequest);
      await client.proveBlock();
      await client.syncState();

      const receiverAccount = await client.getAccount(receiver.id());
      const balance = receiverAccount
        .vault()
        .getBalance(faucet.id())
        .toString();

      return { balance };
    });

    expect(result.balance).toEqual("100");
  });
});

// CUSTOM_TRANSACTIONS TESTS
// =======================================================================================================

test.describe("custom transaction tests", () => {
  test("custom transaction completes successfully", async ({ run }) => {
    await run(async ({ client, sdk }) => {
      const wallet = await client.newWallet(
        sdk.AccountStorageMode.private(),
        false,
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

      // Creating NOTE_ARGS
      const felt1 = new sdk.Felt(sdk.u64(9));
      const felt2 = new sdk.Felt(sdk.u64(12));
      const felt3 = new sdk.Felt(sdk.u64(18));
      const felt4 = new sdk.Felt(sdk.u64(3));
      const felt5 = new sdk.Felt(sdk.u64(3));
      const felt6 = new sdk.Felt(sdk.u64(18));
      const felt7 = new sdk.Felt(sdk.u64(12));
      const felt8 = new sdk.Felt(sdk.u64(9));

      const noteArgs = [felt1, felt2, felt3, felt4, felt5, felt6, felt7, felt8];

      const noteAssets = new sdk.NoteAssets([
        new sdk.FungibleAsset(faucet.id(), sdk.u64(10)),
      ]);

      const noteMetadata = new sdk.NoteMetadata(
        faucet.id(),
        sdk.NoteType.Private,
        sdk.NoteTag.withAccountTarget(wallet.id())
      );

      const expectedNoteArgs = noteArgs.map((felt) => felt.asInt());
      const memAddress = "1000";
      const memAddress2 = "1004";
      const expectedNoteArg1 = expectedNoteArgs.slice(0, 4).join(".");
      const expectedNoteArg2 = expectedNoteArgs.slice(4, 8).join(".");

      const noteScript = `
        # Custom P2ID note script
        #
        # This note script asserts that the note args are exactly the same as passed
        # (currently defined as {expected_note_arg_1} and {expected_note_arg_2}).
        # Since the args are too big to fit in a single note arg, we provide them via advice inputs and
        # address them via their commitment (noted as NOTE_ARG)
        # This note script is based off of the P2ID note script because notes currently need to have
        # assets, otherwise it could have been boiled down to the assert.

        use miden::protocol::active_account
        use miden::protocol::account_id
        use miden::protocol::active_note
        use miden::standards::wallets::basic->basic_wallet
        use miden::core::mem
        begin
            # push data from the advice map into the advice stack
            adv.push_mapval
            # => [NOTE_ARG]

            # memory address where to write the data
            push.${memAddress}
            # => [target_mem_addr, NOTE_ARG_COMMITMENT]
            # number of words
            push.2
            # => [number_of_words, target_mem_addr, NOTE_ARG_COMMITMENT]
            exec.mem::pipe_preimage_to_memory
            # => [target_mem_addr']
            dropw
            # => []

            # read first word
            push.${memAddress}
            # => [data_mem_address]
            mem_loadw_be
            # => [NOTE_ARG_1]

            push.${expectedNoteArg1} assert_eqw.err="First note argument didn't match expected"
            # => []

            # read second word
            push.${memAddress2}
            # => [data_mem_address_2]
            mem_loadw_be
            # => [NOTE_ARG_2]

            push.${expectedNoteArg2} assert_eqw.err="Second note argument didn't match expected"
            # => []

            # store the note storage to memory starting at address 0
            padw push.0 exec.active_note::get_storage
            # => [num_storage_items, storage_ptr, EMPTY_WORD]

            # make sure the number of storage items is 2
            eq.2 assert.err="P2ID script expects exactly 2 note storage items"
            # => [storage_ptr, EMPTY_WORD]

            # read the target account id from the note storage
            mem_loadw_be drop drop
            # => [target_account_id_prefix, target_account_id_suffix]

            exec.active_account::get_id
            # => [account_id_prefix, account_id_suffix, target_account_id_prefix, target_account_id_suffix, ...]

            # ensure account_id = target_account_id, fails otherwise
            exec.account_id::is_equal assert.err="P2ID's target account address and transaction address do not match"
            # => []

            exec.basic_wallet::add_assets_to_account
            # => []
        end
      `;

      const builder = await client.createCodeBuilder();
      const compiledNoteScript = builder.compileNoteScript(noteScript);
      const noteStorage = new sdk.NoteStorage(
        new sdk.FeltArray([wallet.id().suffix(), wallet.id().prefix()])
      );

      const serialNum = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));

      const noteRecipient = new sdk.NoteRecipient(
        serialNum,
        compiledNoteScript,
        noteStorage
      );

      const customNote = new sdk.Note(noteAssets, noteMetadata, noteRecipient);

      // Creating First Custom Transaction Request to Mint the Custom Note
      const outputNote = sdk.OutputNote.full(customNote);
      const transactionRequest = new sdk.TransactionRequestBuilder()
        .withOwnOutputNotes(new sdk.OutputNoteArray([outputNote]))
        .build();

      // Execute and Submit Transaction
      const mintTxId = await client.submitNewTransaction(
        faucet.id(),
        transactionRequest
      );
      await client.proveBlock();
      await client.syncState();

      // Just like in the miden test, you can modify this script to get the execution to fail
      // by modifying the assert (assertedValue = "0" means success)
      const txScript = `
        begin
            push.0 push.0
            # => [0, 0]
            assert_eq
        end
      `;

      // Creating Second Custom Transaction Request to Consume Custom Note
      // with Valid Transaction Script
      const transactionScript = await builder.compileTxScript(txScript);
      const noteArgsCommitment = sdk.Rpo256.hashElements(
        new sdk.FeltArray(noteArgs)
      );

      const noteAndArgs = new sdk.NoteAndArgs(customNote, noteArgsCommitment);

      const adviceMap = new sdk.AdviceMap();
      const noteArgsCommitment2 = sdk.Rpo256.hashElements(
        new sdk.FeltArray(noteArgs)
      );
      adviceMap.insert(noteArgsCommitment2, new sdk.FeltArray(noteArgs));

      const transactionRequest2 = new sdk.TransactionRequestBuilder()
        .withInputNotes(new sdk.NoteAndArgsArray([noteAndArgs]))
        .withCustomScript(transactionScript)
        .extendAdviceMap(adviceMap)
        .build();

      // Execute and Submit Transaction
      await client.submitNewTransaction(wallet.id(), transactionRequest2);
      await client.proveBlock();
      await client.syncState();

      return {};
    });
  });

  test("custom transaction fails with invalid assert", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
      const wallet = await client.newWallet(
        sdk.AccountStorageMode.private(),
        false,
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

      // Creating NOTE_ARGS
      const noteArgs = [
        new sdk.Felt(sdk.u64(9)),
        new sdk.Felt(sdk.u64(12)),
        new sdk.Felt(sdk.u64(18)),
        new sdk.Felt(sdk.u64(3)),
        new sdk.Felt(sdk.u64(3)),
        new sdk.Felt(sdk.u64(18)),
        new sdk.Felt(sdk.u64(12)),
        new sdk.Felt(sdk.u64(9)),
      ];

      const noteAssets = new sdk.NoteAssets([
        new sdk.FungibleAsset(faucet.id(), sdk.u64(10)),
      ]);

      const noteMetadata = new sdk.NoteMetadata(
        faucet.id(),
        sdk.NoteType.Private,
        sdk.NoteTag.withAccountTarget(wallet.id())
      );

      const expectedNoteArgs = noteArgs.map((felt) => felt.asInt());
      const memAddress = "1000";
      const memAddress2 = "1004";
      const expectedNoteArg1 = expectedNoteArgs.slice(0, 4).join(".");
      const expectedNoteArg2 = expectedNoteArgs.slice(4, 8).join(".");

      const noteScript = `
        use miden::protocol::active_account
        use miden::protocol::account_id
        use miden::protocol::active_note
        use miden::standards::wallets::basic->basic_wallet
        use miden::core::mem
        begin
            adv.push_mapval
            push.${memAddress}
            push.2
            exec.mem::pipe_preimage_to_memory
            dropw
            push.${memAddress}
            mem_loadw_be
            push.${expectedNoteArg1} assert_eqw.err="First note argument didn't match expected"
            push.${memAddress2}
            mem_loadw_be
            push.${expectedNoteArg2} assert_eqw.err="Second note argument didn't match expected"
            padw push.0 exec.active_note::get_storage
            eq.2 assert.err="P2ID script expects exactly 2 note storage items"
            mem_loadw_be drop drop
            exec.active_account::get_id
            exec.account_id::is_equal assert.err="P2ID's target account address and transaction address do not match"
            exec.basic_wallet::add_assets_to_account
        end
      `;

      const builder = await client.createCodeBuilder();
      const compiledNoteScript = builder.compileNoteScript(noteScript);
      const noteStorage = new sdk.NoteStorage(
        new sdk.FeltArray([wallet.id().suffix(), wallet.id().prefix()])
      );

      const serialNum = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));
      const noteRecipient = new sdk.NoteRecipient(
        serialNum,
        compiledNoteScript,
        noteStorage
      );
      const customNote = new sdk.Note(noteAssets, noteMetadata, noteRecipient);

      const outputNote = sdk.OutputNote.full(customNote);
      const transactionRequest = new sdk.TransactionRequestBuilder()
        .withOwnOutputNotes(new sdk.OutputNoteArray([outputNote]))
        .build();

      await client.submitNewTransaction(faucet.id(), transactionRequest);
      await client.proveBlock();
      await client.syncState();

      // Failing tx script: asserts 0 == 1
      const txScript = `
        begin
            push.0 push.1
            # => [0, 1]
            assert_eq
        end
      `;

      const transactionScript = await builder.compileTxScript(txScript);
      const noteArgsCommitment = sdk.Rpo256.hashElements(
        new sdk.FeltArray(noteArgs)
      );

      const noteAndArgs = new sdk.NoteAndArgs(customNote, noteArgsCommitment);

      const adviceMap = new sdk.AdviceMap();
      const noteArgsCommitment2 = sdk.Rpo256.hashElements(
        new sdk.FeltArray(noteArgs)
      );
      adviceMap.insert(noteArgsCommitment2, new sdk.FeltArray(noteArgs));

      const transactionRequest2 = new sdk.TransactionRequestBuilder()
        .withInputNotes(new sdk.NoteAndArgsArray([noteAndArgs]))
        .withCustomScript(transactionScript)
        .extendAdviceMap(adviceMap)
        .build();

      let threw = false;
      try {
        await client.submitNewTransaction(wallet.id(), transactionRequest2);
      } catch {
        threw = true;
      }

      return { threw };
    });

    expect(result.threw).toBe(true);
  });
});

test.describe("custom transaction with multiple output notes", () => {
  test("does not fail when output note serial numbers are unique", async ({
    run,
  }) => {
    await run(async ({ client, sdk, helpers }) => {
      const { wallet: sender, faucet } = await helpers.setupWalletAndFaucet();
      await helpers.mockMintAndConsume(sender.id(), faucet.id());

      const amount = sdk.u64(10);
      const target = await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );

      const noteAssets1 = new sdk.NoteAssets([
        new sdk.FungibleAsset(faucet.id(), amount),
      ]);
      const noteAssets2 = new sdk.NoteAssets([
        new sdk.FungibleAsset(faucet.id(), amount),
      ]);

      const noteMetadata = new sdk.NoteMetadata(
        sender.id(),
        sdk.NoteType.Public,
        sdk.NoteTag.withAccountTarget(target.id())
      );

      const serialNum1 = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));
      const serialNum2 = new sdk.Word(sdk.u64Array([5, 6, 7, 8]));

      const p2idScript = sdk.NoteScript.p2id();

      const noteStorage = new sdk.NoteStorage(
        new sdk.FeltArray([target.id().suffix(), target.id().prefix()])
      );

      const noteRecipient1 = new sdk.NoteRecipient(
        serialNum1,
        p2idScript,
        noteStorage
      );
      const noteRecipient2 = new sdk.NoteRecipient(
        serialNum2,
        p2idScript,
        noteStorage
      );

      const note1 = new sdk.Note(noteAssets1, noteMetadata, noteRecipient1);
      const note2 = new sdk.Note(noteAssets2, noteMetadata, noteRecipient2);

      const outputNotes = [
        sdk.OutputNote.full(note1),
        sdk.OutputNote.full(note2),
      ];

      const transactionRequest = new sdk.TransactionRequestBuilder()
        .withOwnOutputNotes(new sdk.OutputNoteArray(outputNotes))
        .build();

      await client.submitNewTransaction(sender.id(), transactionRequest);
      await client.proveBlock();
      await client.syncState();

      return {};
    });
  });

  test("fails when output note serial numbers are the same", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk, helpers }) => {
      const { wallet: sender, faucet } = await helpers.setupWalletAndFaucet();
      await helpers.mockMintAndConsume(sender.id(), faucet.id());

      const amount = sdk.u64(10);
      const target = await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );

      const noteAssets1 = new sdk.NoteAssets([
        new sdk.FungibleAsset(faucet.id(), amount),
      ]);
      const noteAssets2 = new sdk.NoteAssets([
        new sdk.FungibleAsset(faucet.id(), amount),
      ]);

      const noteMetadata = new sdk.NoteMetadata(
        sender.id(),
        sdk.NoteType.Public,
        sdk.NoteTag.withAccountTarget(target.id())
      );

      const serialNum1 = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));

      const p2idScript = sdk.NoteScript.p2id();

      const noteStorage = new sdk.NoteStorage(
        new sdk.FeltArray([target.id().suffix(), target.id().prefix()])
      );

      const noteRecipient1 = new sdk.NoteRecipient(
        serialNum1,
        p2idScript,
        noteStorage
      );
      // Same serial number for second note
      const noteRecipient2 = new sdk.NoteRecipient(
        serialNum1,
        p2idScript,
        noteStorage
      );

      const note1 = new sdk.Note(noteAssets1, noteMetadata, noteRecipient1);
      const note2 = new sdk.Note(noteAssets2, noteMetadata, noteRecipient2);

      const outputNotes = [
        sdk.OutputNote.full(note1),
        sdk.OutputNote.full(note2),
      ];

      const transactionRequest = new sdk.TransactionRequestBuilder()
        .withOwnOutputNotes(new sdk.OutputNoteArray(outputNotes))
        .build();

      let threw = false;
      try {
        await client.submitNewTransaction(sender.id(), transactionRequest);
      } catch {
        threw = true;
      }

      return { threw };
    });

    expect(result.threw).toBe(true);
  });
});

// CUSTOM ACCOUNT COMPONENT TESTS
// =======================================================================================================

test.describe("custom account component tests", () => {
  test("custom account component transaction completes successfully (ECDSA)", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk }) => {
      const MAP_SLOT_NAME =
        "miden::testing::mapping_example_contract::map_slot";

      const accountCode = `
        use miden::protocol::active_account
        use miden::protocol::native_account
        use miden::core::word
        use miden::core::sys

        const MAP_SLOT = word("${MAP_SLOT_NAME}")

        # Inputs: [KEY, VALUE]
        # Outputs: []
        pub proc write_to_map
            # Setting the key value pair in the map
            push.MAP_SLOT[0..2]
            exec.native_account::set_map_item
            # => [OLD_MAP_ROOT, OLD_MAP_VALUE]

            dropw dropw dropw dropw
            # => []
        end

        # Inputs: [KEY]
        # Outputs: [VALUE]
        pub proc get_value_in_map
            push.MAP_SLOT[0..2]
            exec.active_account::get_map_item
            # => [VALUE]
        end

        # Inputs: []
        # Outputs: [CURRENT_ROOT]
        pub proc get_current_map_root
            push.MAP_SLOT[0..2] exec.active_account::get_item
            # => [CURRENT_ROOT]

            exec.sys::truncate_stack
            # => [CURRENT_ROOT]
        end
      `;
      const scriptCode = `
        use miden_by_example::mapping_example_contract
        use miden::core::sys

        begin
            push.1.2.3.4
            push.0.0.0.0
            # => [KEY, VALUE]

            call.mapping_example_contract::write_to_map
            # => []

            push.0.0.0.0
            # => [KEY]

            call.mapping_example_contract::get_value_in_map
            # => [VALUE]

            dropw
            # => []

            call.mapping_example_contract::get_current_map_root
            # => [CURRENT_ROOT]

            exec.sys::truncate_stack
        end
      `;

      const builder = await client.createCodeBuilder();
      const storageMap = new sdk.StorageMap();
      const storageSlotMap = sdk.StorageSlot.map(MAP_SLOT_NAME, storageMap);

      const accountComponentCode =
        builder.compileAccountComponentCode(accountCode);
      const mappingAccountComponent = sdk.AccountComponent.compile(
        accountComponentCode,
        [storageSlotMap]
      ).withSupportsAllTypes();

      const walletSeed = new Uint8Array(32);
      crypto.getRandomValues(walletSeed);

      const secretKey = sdk.AuthSecretKey.ecdsaWithRNG(walletSeed);
      const authComponent =
        sdk.AccountComponent.createAuthComponentFromSecretKey(secretKey);

      const accountBuilderResult = new sdk.AccountBuilder(walletSeed)
        .accountType(2 /* RegularAccountImmutableCode */)
        .storageMode(sdk.AccountStorageMode.public())
        .withAuthComponent(authComponent)
        .withComponent(mappingAccountComponent)
        .build();

      await client.addAccountSecretKeyToWebStore(
        accountBuilderResult.account.id(),
        secretKey
      );
      await client.newAccount(accountBuilderResult.account, false);

      const accountCodeLib = builder.buildLibrary(
        "miden_by_example::mapping_example_contract",
        accountCode
      );

      builder.linkStaticLibrary(accountCodeLib);

      const txScript = builder.compileTxScript(scriptCode);

      const txIncrementRequest = new sdk.TransactionRequestBuilder()
        .withCustomScript(txScript)
        .build();

      await client.submitNewTransaction(
        accountBuilderResult.account.id(),
        txIncrementRequest
      );
      await client.proveBlock();
      await client.syncState();

      // Fetch the updated account state from the client
      const updated = await client.getAccount(
        accountBuilderResult.account.id()
      );

      // Read a map value from storage slot with key 0x0
      const keyZero = new sdk.Word(sdk.u64Array([0, 0, 0, 0]));
      const retrieveMapKey = updated
        ?.storage()
        .getMapItem(MAP_SLOT_NAME, keyZero);

      const expected = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));

      return {
        retrieveMapKeyHex: retrieveMapKey?.toHex(),
        expectedHex: expected.toHex(),
      };
    });

    expect(result.retrieveMapKeyHex).toEqual(result.expectedHex);
  });

  test("custom account component transaction completes successfully (Falcon)", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk }) => {
      const MAP_SLOT_NAME =
        "miden::testing::mapping_example_contract::map_slot";

      const accountCode = `
        use miden::protocol::active_account
        use miden::protocol::native_account
        use miden::core::word
        use miden::core::sys

        const MAP_SLOT = word("${MAP_SLOT_NAME}")

        # Inputs: [KEY, VALUE]
        # Outputs: []
        pub proc write_to_map
            # Setting the key value pair in the map
            push.MAP_SLOT[0..2]
            exec.native_account::set_map_item
            # => [OLD_MAP_ROOT, OLD_MAP_VALUE]

            dropw dropw dropw dropw
            # => []
        end

        # Inputs: [KEY]
        # Outputs: [VALUE]
        pub proc get_value_in_map
            push.MAP_SLOT[0..2]
            exec.active_account::get_map_item
            # => [VALUE]
        end

        # Inputs: []
        # Outputs: [CURRENT_ROOT]
        pub proc get_current_map_root
            push.MAP_SLOT[0..2] exec.active_account::get_item
            # => [CURRENT_ROOT]

            exec.sys::truncate_stack
            # => [CURRENT_ROOT]
        end
      `;
      const scriptCode = `
        use miden_by_example::mapping_example_contract
        use miden::core::sys

        begin
            push.1.2.3.4
            push.0.0.0.0
            # => [KEY, VALUE]

            call.mapping_example_contract::write_to_map
            # => []

            push.0.0.0.0
            # => [KEY]

            call.mapping_example_contract::get_value_in_map
            # => [VALUE]

            dropw
            # => []

            call.mapping_example_contract::get_current_map_root
            # => [CURRENT_ROOT]

            exec.sys::truncate_stack
        end
      `;

      const builder = await client.createCodeBuilder();
      const storageMap = new sdk.StorageMap();
      const storageSlotMap = sdk.StorageSlot.map(MAP_SLOT_NAME, storageMap);

      const accountComponentCode =
        builder.compileAccountComponentCode(accountCode);
      const mappingAccountComponent = sdk.AccountComponent.compile(
        accountComponentCode,
        [storageSlotMap]
      ).withSupportsAllTypes();

      const walletSeed = new Uint8Array(32);
      crypto.getRandomValues(walletSeed);

      const secretKey = sdk.AuthSecretKey.rpoFalconWithRNG(walletSeed);
      const authComponent =
        sdk.AccountComponent.createAuthComponentFromSecretKey(secretKey);

      const accountBuilderResult = new sdk.AccountBuilder(walletSeed)
        .accountType(2 /* RegularAccountImmutableCode */)
        .storageMode(sdk.AccountStorageMode.public())
        .withAuthComponent(authComponent)
        .withComponent(mappingAccountComponent)
        .build();

      await client.addAccountSecretKeyToWebStore(
        accountBuilderResult.account.id(),
        secretKey
      );
      await client.newAccount(accountBuilderResult.account, false);

      const accountCodeLib = builder.buildLibrary(
        "miden_by_example::mapping_example_contract",
        accountCode
      );

      builder.linkStaticLibrary(accountCodeLib);

      const txScript = builder.compileTxScript(scriptCode);

      const txIncrementRequest = new sdk.TransactionRequestBuilder()
        .withCustomScript(txScript)
        .build();

      await client.submitNewTransaction(
        accountBuilderResult.account.id(),
        txIncrementRequest
      );
      await client.proveBlock();
      await client.syncState();

      // Fetch the updated account state from the client
      const updated = await client.getAccount(
        accountBuilderResult.account.id()
      );

      // Read a map value from storage slot with key 0x0
      const keyZero = new sdk.Word(sdk.u64Array([0, 0, 0, 0]));
      const retrieveMapKey = updated
        ?.storage()
        .getMapItem(MAP_SLOT_NAME, keyZero);

      const expected = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));

      return {
        retrieveMapKeyHex: retrieveMapKey?.toHex(),
        expectedHex: expected.toHex(),
      };
    });

    expect(result.retrieveMapKeyHex).toEqual(result.expectedHex);
  });
});

// DISCARDED TRANSACTIONS TESTS
// ================================================================================================
// Skipped: requires exportStore/importStore which are browser-only (IndexedDB)

// NETWORK TRANSACTION TESTS
// ================================================================================================
// Skipped: requires a running node (waitForBlocks, network account)

// STORAGE MAP TEST
// =======================================================================================================

test.describe("storage map test", () => {
  test("storage map is updated correctly in transaction", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
      const MAP_SLOT_NAME = "miden::testing::bump_item_contract::map_slot";

      const normalizeHexWord = (hex) => {
        if (!hex) return undefined;
        const normalized = hex.replace(/^0x/, "").replace(/^0+|0+$/g, "");
        return normalized;
      };

      // BUILD ACCOUNT WITH COMPONENT THAT MODIFIES STORAGE MAP
      const MAP_KEY = new sdk.Word(sdk.u64Array([1, 1, 1, 1]));
      const FPI_STORAGE_VALUE = new sdk.Word(sdk.u64Array([0, 0, 0, 1]));

      const storageMap = new sdk.StorageMap();
      storageMap.insert(MAP_KEY, FPI_STORAGE_VALUE);
      storageMap.insert(
        new sdk.Word(sdk.u64Array([2, 2, 2, 2])),
        new sdk.Word(sdk.u64Array([0, 0, 0, 9]))
      );

      const accountCode = `
        use miden::core::word

        const MAP_SLOT = word("${MAP_SLOT_NAME}")

        pub proc bump_map_item
        # map key
        push.1.1.1.1 # Map key
        push.MAP_SLOT[0..2]
        exec.::miden::protocol::active_account::get_map_item
        add.1
        push.1.1.1.1 # Map key
        push.MAP_SLOT[0..2]
        exec.::miden::protocol::native_account::set_map_item
        # => [OLD_MAP_ROOT, OLD_VALUE]
        dropw dropw
        end
      `;

      const builder = await client.createCodeBuilder();
      const accountComponentCode =
        builder.compileAccountComponentCode(accountCode);
      const bumpItemComponent = sdk.AccountComponent.compile(
        accountComponentCode,
        [sdk.StorageSlot.map(MAP_SLOT_NAME, storageMap)]
      ).withSupportsAllTypes();

      const walletSeed = new Uint8Array(32);
      crypto.getRandomValues(walletSeed);

      const secretKey = sdk.AuthSecretKey.rpoFalconWithRNG(walletSeed);
      const authComponent =
        sdk.AccountComponent.createAuthComponentFromSecretKey(secretKey);

      const bumpItemAccountBuilderResult = new sdk.AccountBuilder(walletSeed)
        .withAuthComponent(authComponent)
        .withComponent(bumpItemComponent)
        .storageMode(sdk.AccountStorageMode.public())
        .build();

      await client.addAccountSecretKeyToWebStore(
        bumpItemAccountBuilderResult.account.id(),
        secretKey
      );
      await client.newAccount(bumpItemAccountBuilderResult.account, false);

      const initialMapValue = (
        await client.getAccount(bumpItemAccountBuilderResult.account.id())
      )
        ?.storage()
        .getMapItem(MAP_SLOT_NAME, MAP_KEY)
        ?.toHex();

      const accountComponentLib = builder.buildLibrary(
        "external_contract::bump_item_contract",
        accountCode
      );

      builder.linkDynamicLibrary(accountComponentLib);

      const txScript = builder.compileTxScript(
        `use external_contract::bump_item_contract
        begin
            call.bump_item_contract::bump_map_item
        end`
      );

      const txIncrementRequest = new sdk.TransactionRequestBuilder()
        .withCustomScript(txScript)
        .build();

      await client.submitNewTransaction(
        bumpItemAccountBuilderResult.account.id(),
        txIncrementRequest
      );
      await client.proveBlock();
      await client.syncState();

      const finalMapValue = (
        await client.getAccount(bumpItemAccountBuilderResult.account.id())
      )
        ?.storage()
        .getMapItem(MAP_SLOT_NAME, MAP_KEY)
        ?.toHex();

      // Test getMapEntries() functionality
      const accountStorage = (
        await client.getAccount(bumpItemAccountBuilderResult.account.id())
      )?.storage();
      const mapEntries = accountStorage?.getMapEntries(MAP_SLOT_NAME);

      const normalizedInitial = normalizeHexWord(initialMapValue);
      const normalizedFinal = normalizeHexWord(finalMapValue);

      const mapKeyHex = MAP_KEY.toHex();
      const hasExpectedEntry = mapEntries?.some(
        (entry) =>
          entry.key === mapKeyHex &&
          normalizeHexWord(entry.value) === normalizedFinal
      );

      return {
        normalizedInitial,
        normalizedFinal,
        mapEntriesLength: mapEntries?.length,
        hasExpectedEntry,
      };
    });

    expect(result.normalizedInitial).toBe("1");
    expect(result.normalizedFinal).toBe("2");
    expect(result.mapEntriesLength).toBeGreaterThan(1);
    expect(result.hasExpectedEntry).toBe(true);
  });
});

// SUBMIT_NEW_TRANSACTION_WITH_PROVER TESTS
// ================================================================================================

test.describe("submitNewTransactionWithProver tests", () => {
  test("submitNewTransactionWithProver with failing prover throws", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk, helpers }) => {
      const { wallet, faucet } = await helpers.setupWalletAndFaucet();

      const mintTransactionRequest = await client.newMintTransactionRequest(
        wallet.id(),
        faucet.id(),
        sdk.NoteType.Public,
        sdk.u64(1000)
      );

      // Create a failing remote prover with an invalid endpoint
      const failingProver = sdk.TransactionProver.newRemoteProver(
        "http://localhost:1",
        null
      );

      let threw = false;
      try {
        await client.submitNewTransactionWithProver(
          faucet.id(),
          mintTransactionRequest,
          failingProver
        );
      } catch {
        threw = true;
      }

      return { threw };
    });

    expect(result.threw).toBe(true);
  });

  test.describe("executeForSummary tests", () => {
    test("executeForSummary returns TransactionSummary for unauthorized transaction", async ({
      run,
    }) => {
      const result = await run(async ({ client, sdk }) => {
        const walletSeed = new Uint8Array(32);
        crypto.getRandomValues(walletSeed);

        const approverKeys = [
          sdk.AuthSecretKey.rpoFalconWithRNG(),
          sdk.AuthSecretKey.rpoFalconWithRNG(),
          sdk.AuthSecretKey.rpoFalconWithRNG(),
        ];
        const approverCommitments = approverKeys.map((key) =>
          key.publicKey().toCommitment()
        );
        const multisigConfig = new sdk.AuthFalcon512RpoMultisigConfig(
          approverCommitments,
          2
        );
        const multisigComponent =
          sdk.createAuthFalcon512RpoMultisig(multisigConfig);

        const accountBuilderResult = new sdk.AccountBuilder(walletSeed)
          .accountType(2 /* RegularAccountImmutableCode */)
          .storageMode(sdk.AccountStorageMode.private())
          .withAuthComponent(multisigComponent)
          .withBasicWalletComponent()
          .build();

        const multisigAccountId = accountBuilderResult.account.id();
        await client.newAccount(accountBuilderResult.account, false);

        // Register the approver keys with the multisig account
        for (const key of approverKeys) {
          await client.addAccountSecretKeyToWebStore(multisigAccountId, key);
        }

        const targetAccount = await client.newWallet(
          sdk.AccountStorageMode.private(),
          false,
          sdk.AuthScheme.AuthRpoFalcon512
        );

        const faucetAccount = await client.newFaucet(
          sdk.AccountStorageMode.private(),
          false,
          "DAG",
          8,
          sdk.u64(10000000),
          sdk.AuthScheme.AuthRpoFalcon512
        );

        // Mint and consume to fund target
        const mintRequest = await client.newMintTransactionRequest(
          targetAccount.id(),
          faucetAccount.id(),
          sdk.NoteType.Public,
          sdk.u64(1000)
        );
        const mintTxId = await client.submitNewTransaction(
          faucetAccount.id(),
          mintRequest
        );
        await client.proveBlock();
        await client.syncState();

        const [mintTxRecord] = await client.getTransactions(
          sdk.TransactionFilter.ids([mintTxId])
        );
        const createdNoteIds = mintTxRecord
          .outputNotes()
          .notes()
          .map((note) => note.id().toString());

        // Convert note IDs to Note objects for consume request
        const createdNotes = await Promise.all(
          createdNoteIds.map(async (noteId) => {
            const inputNoteRecord = await client.getInputNote(noteId);
            if (!inputNoteRecord) {
              throw new Error(`Note with ID ${noteId} not found`);
            }
            return inputNoteRecord.toNote();
          })
        );

        const consumeTransactionRequest =
          client.newConsumeTransactionRequest(createdNotes);

        await client.submitNewTransaction(
          targetAccount.id(),
          consumeTransactionRequest
        );
        await client.proveBlock();
        await client.syncState();

        const sendTransactionRequest = await client.newSendTransactionRequest(
          targetAccount.id(),
          accountBuilderResult.account.id(),
          faucetAccount.id(),
          sdk.NoteType.Public,
          sdk.u64(100),
          null,
          null
        );

        const sendTxId = await client.submitNewTransaction(
          targetAccount.id(),
          sendTransactionRequest
        );
        await client.proveBlock();
        await client.syncState();

        const [sendTxRecord] = await client.getTransactions(
          sdk.TransactionFilter.ids([sendTxId])
        );
        const sentNoteIds = sendTxRecord
          .outputNotes()
          .notes()
          .map((note) => note.id().toString());

        // Convert note IDs to Note objects for consume request
        const sentNotes = await Promise.all(
          sentNoteIds.map(async (noteId) => {
            const inputNoteRecord = await client.getInputNote(noteId);
            if (!inputNoteRecord) {
              throw new Error(`Note with ID ${noteId} not found`);
            }
            return inputNoteRecord.toNote();
          })
        );

        const consumeSentNoteRequest =
          client.newConsumeTransactionRequest(sentNotes);

        const summary = await client.executeForSummary(
          accountBuilderResult.account.id(),
          consumeSentNoteRequest
        );

        const summaryInputNoteIds = summary
          .inputNotes()
          .notes()
          .map((note) => note.id().toString());

        return {
          inputNotesCount: summary.inputNotes().numNotes(),
          outputNotesCount: summary.outputNotes().numNotes(),
          summaryInputNoteIds,
          sentNoteIds,
        };
      });

      expect(result.inputNotesCount).toBe(1);
      expect(result.outputNotesCount).toBe(0);
      expect(result.summaryInputNoteIds).toEqual(result.sentNoteIds);
    });

    test("executeForSummary returns TransactionSummary for authorized transaction with matching salt", async ({
      run,
    }) => {
      const result = await run(async ({ client, sdk }) => {
        const senderAccount = await client.newWallet(
          sdk.AccountStorageMode.private(),
          false,
          sdk.AuthScheme.AuthRpoFalcon512
        );

        // Create a known salt value
        const expectedSalt = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));

        // Build transaction request with the salt as auth_arg
        const transactionRequest = new sdk.TransactionRequestBuilder()
          .withAuthArg(expectedSalt)
          .build();

        const summary = await client.executeForSummary(
          senderAccount.id(),
          transactionRequest
        );

        return {
          inputNotesCount: summary.inputNotes().numNotes(),
          outputNotesCount: summary.outputNotes().numNotes(),
          saltHex: summary.salt().toHex(),
          expectedSaltHex: expectedSalt.toHex(),
        };
      });

      expect(result.inputNotesCount).toBe(0);
      expect(result.outputNotesCount).toBe(0);
      expect(result.saltHex).toBe(result.expectedSaltHex);
    });
  });
});
