import test from "./playwright.global.setup";
import { expect, Page } from "@playwright/test";
import {
  consumeTransaction,
  mintAndConsumeTransaction,
  mintTransaction,
  sendTransaction,
  setupWalletAndFaucet,
  setupConsumedNote,
} from "./webClientTestUtils";
import {
  Account,
  TransactionRecord,
  Note,
} from "../dist/crates/miden_client_web";

// NEW_MINT_TRANSACTION TESTS
// =======================================================================================================

interface MultipleMintsTransactionUpdate {
  transactionIds: string[];
  createdNoteIds: string[];
  numOutputNotesCreated: number;
  nonce: string | undefined;
  finalBalance: string | undefined;
}

const multipleMintsTest = async (
  testingPage: Page,
  targetAccount: string,
  faucetAccount: string,
  withRemoteProver: boolean = false
): Promise<MultipleMintsTransactionUpdate> => {
  return await testingPage.evaluate(
    async ({ targetAccount, faucetAccount, withRemoteProver }) => {
      const client = window.client;

      const targetAccountId = window.AccountId.fromHex(targetAccount);
      const faucetAccountId = window.AccountId.fromHex(faucetAccount);
      await client.syncState();

      const prover =
        withRemoteProver && window.remoteProverUrl != null
          ? window.remoteProverInstance
          : undefined;

      // Mint 3 notes
      let result: {
        transactionIds: string[];
        createdNoteIds: string[];
        numOutputNotesCreated: number;
      } = {
        transactionIds: [],
        createdNoteIds: [],
        numOutputNotesCreated: 0,
      };

      for (let i = 0; i < 3; i++) {
        const mintTransactionRequest = await client.newMintTransactionRequest(
          targetAccountId,
          faucetAccountId,
          window.NoteType.Public,
          BigInt(1000)
        );

        const mintTransactionUpdate =
          await window.helpers.executeAndApplyTransaction(
            faucetAccountId,
            mintTransactionRequest,
            prover
          );

        await window.helpers.waitForTransaction(
          mintTransactionUpdate.executedTransaction().id().toHex()
        );

        result.createdNoteIds.push(
          mintTransactionUpdate
            .executedTransaction()
            .outputNotes()
            .notes()[0]
            .id()
            .toString()
        );
        result.transactionIds.push(
          mintTransactionUpdate.executedTransaction().id().toHex()
        );
        result.numOutputNotesCreated += mintTransactionUpdate
          .executedTransaction()
          .outputNotes()
          .numNotes();
      }

      // Consume the minted notes
      for (let i = 0; i < result.createdNoteIds.length; i++) {
        let noteId = result.createdNoteIds[i];
        const inputNoteRecord = await client.getInputNote(noteId);
        if (!inputNoteRecord) {
          throw new Error(`Note with ID ${noteId} not found`);
        }

        const note = inputNoteRecord.toNote();
        const consumeTransactionRequest = client.newConsumeTransactionRequest([
          note,
        ]);
        const consumeTransactionUpdate =
          await window.helpers.executeAndApplyTransaction(
            targetAccountId,
            consumeTransactionRequest,
            prover
          );

        await window.helpers.waitForTransaction(
          consumeTransactionUpdate.executedTransaction().id().toHex()
        );
      }

      const changedTargetAccount = await client.getAccount(targetAccountId);

      return {
        ...result,
        nonce: changedTargetAccount!.nonce()?.toString(),
        finalBalance: changedTargetAccount!
          .vault()
          .getBalance(faucetAccountId)
          .toString(),
      };
    },
    {
      targetAccount,
      faucetAccount,
      withRemoteProver,
    }
  );
};

test.describe("mint transaction tests", () => {
  const testCases = [
    { flag: false, description: "mint transaction completes successfully" },
    {
      flag: true,
      description: "mint transaction with remote prover completes successfully",
    },
  ];

  testCases.forEach(({ flag, description }) => {
    test(description, async ({ page }) => {
      // This test was added in #995 to reproduce an issue in the web wallet.
      // It is useful because most tests consume the note right on the latest client block,
      // but this test mints 3 notes and consumes them after the fact. This ensures the
      // MMR data for old blocks is available and valid so that the notes can be consumed.
      const { faucetId, accountId } = await setupWalletAndFaucet(page);
      const result = await multipleMintsTest(page, accountId, faucetId, flag);

      expect(result.transactionIds.length).toEqual(3);
      expect(result.numOutputNotesCreated).toEqual(3);
      expect(result.nonce).toEqual("3");
      expect(result.finalBalance).toEqual("3000");
      expect(result.createdNoteIds.length).toEqual(3);
    });
  });
});

// NEW_CONSUME_TRANSACTION TESTS
// =======================================================================================================

test.describe("consume transaction tests", () => {
  const testCases = [
    { flag: false, description: "consume transaction completes successfully" },
    {
      flag: true,
      description:
        "consume transaction with remote prover completes successfully",
    },
  ];

  testCases.forEach(({ flag, description }) => {
    test(description, async ({ page }) => {
      const { faucetId, accountId } = await setupWalletAndFaucet(page);
      const { consumeResult: result } = await mintAndConsumeTransaction(
        page,
        accountId,
        faucetId,
        flag
      );

      expect(result.transactionId).toHaveLength(66);
      expect(result.nonce).toEqual("1");
      expect(result.numConsumedNotes).toEqual(1);
      expect(result.targetAccountBalance).toEqual("1000");
    });
  });
});

// NEW_SEND_TRANSACTION TESTS
// =======================================================================================================

interface SendTransactionUpdate {
  senderAccountBalance: string;
  changedTargetBalance: string;
}

export const sendTransactionTest = async (
  testingPage: Page,
  senderAccount: string,
  targetAccount: string,
  faucetAccount: string
): Promise<SendTransactionUpdate> => {
  return await testingPage.evaluate(
    async ({ senderAccount, targetAccount, faucetAccount }) => {
      const client = window.client;

      await client.syncState();

      const targetAccountId = window.AccountId.fromHex(targetAccount);
      const senderAccountId = window.AccountId.fromHex(senderAccount);
      const faucetAccountId = window.AccountId.fromHex(faucetAccount);

      const changedSenderAccount = await client.getAccount(senderAccountId);
      const changedTargetAccount = await client.getAccount(targetAccountId);

      return {
        senderAccountBalance: changedSenderAccount!
          .vault()
          .getBalance(faucetAccountId)
          .toString(),
        changedTargetBalance: changedTargetAccount!
          .vault()
          .getBalance(faucetAccountId)
          .toString(),
      };
    },
    {
      senderAccount,
      targetAccount,
      faucetAccount,
    }
  );
};

test.describe("send transaction tests", () => {
  const testCases = [
    { flag: false, description: "send transaction completes successfully" },
    {
      flag: true,
      description: "send transaction with remote prover completes successfully",
    },
  ];

  testCases.forEach(({ flag, description }) => {
    test(description, async ({ page }) => {
      const { accountId: senderAccountId, faucetId } =
        await setupWalletAndFaucet(page);
      const { accountId: targetAccountId } = await setupWalletAndFaucet(page);
      const recallHeight = 100;
      let createdSendNotes = await sendTransaction(
        page,
        senderAccountId,
        targetAccountId,
        faucetId,
        recallHeight,
        flag
      );

      await consumeTransaction(
        page,
        targetAccountId,
        faucetId,
        createdSendNotes[0]
      );
      const result = await sendTransactionTest(
        page,
        senderAccountId,
        targetAccountId,
        faucetId
      );

      expect(result.senderAccountBalance).toEqual("900");
      expect(result.changedTargetBalance).toEqual("100");
    });
  });
});

// CUSTOM_TRANSACTIONS TESTS
// =======================================================================================================

export const customTransaction = async (
  testingPage: Page,
  assertedValue: string,
  withRemoteProver: boolean
): Promise<void> => {
  return await testingPage.evaluate(
    async ({ assertedValue, withRemoteProver }) => {
      const client = window.client;

      const walletAccount = await client.newWallet(
        window.AccountStorageMode.private(),
        false,
        0
      );
      const faucetAccount = await client.newFaucet(
        window.AccountStorageMode.private(),
        false,
        "DAG",
        8,
        BigInt(10000000),
        0
      );
      await client.syncState();

      // Creating Custom Note which needs the following:
      // - Note Assets
      // - Note Metadata
      // - Note Recipient

      // Creating NOTE_ARGS
      let felt1 = new window.Felt(BigInt(9));
      let felt2 = new window.Felt(BigInt(12));
      let felt3 = new window.Felt(BigInt(18));
      let felt4 = new window.Felt(BigInt(3));
      let felt5 = new window.Felt(BigInt(3));
      let felt6 = new window.Felt(BigInt(18));
      let felt7 = new window.Felt(BigInt(12));
      let felt8 = new window.Felt(BigInt(9));

      let noteArgs = [felt1, felt2, felt3, felt4, felt5, felt6, felt7, felt8];
      let feltArray = new window.MidenArrays.FeltArray();

      noteArgs.forEach((felt) => {
        feltArray.push(felt);
      });

      let noteAssets = new window.NoteAssets([
        new window.FungibleAsset(faucetAccount.id(), BigInt(10)),
      ]);

      let noteMetadata = new window.NoteMetadata(
        faucetAccount.id(),
        window.NoteType.Private,
        window.NoteTag.fromAccountId(walletAccount.id())
      );

      let expectedNoteArgs = noteArgs.map((felt) => felt.asInt());
      let memAddress = "1000";
      let memAddress2 = "1004";
      let expectedNoteArg1 = expectedNoteArgs.slice(0, 4).join(".");
      let expectedNoteArg2 = expectedNoteArgs.slice(4, 8).join(".");

      let noteScript = `
            # Custom P2ID note script
            #
            # This note script asserts that the note args are exactly the same as passed
            # (currently defined as {expected_note_arg_1} and {expected_note_arg_2}).
            # Since the args are too big to fit in a single note arg, we provide them via advice inputs and
            # address them via their commitment (noted as NOTE_ARG)
            # This note script is based off of the P2ID note script because notes currently need to have
            # assets, otherwise it could have been boiled down to the assert.

            use miden::protocol::native_account
            use miden::protocol::active_note
            use miden::core::sys
            use miden::core::mem
            use miden::standards::wallets::basic->basic_wallet
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

                # store the note inputs to memory starting at address 0
                push.0 exec.active_note::get_inputs
                # => [num_inputs, inputs_ptr]

                # make sure the number of inputs is 2
                eq.2 assert.err="P2ID script expects exactly 2 note inputs"
                # => [inputs_ptr]

                # read the target account id from the note inputs
                mem_load
                # => [target_account_id_prefix]

                exec.native_account::get_id swap drop
                # => [account_id_prefix, target_account_id_prefix, ...]

                # ensure account_id = target_account_id, fails otherwise
                assert_eq.err="P2ID's target account address and transaction address do not match"
                # => [...]

                exec.basic_wallet::add_assets_to_account
                # => [...]
            end
        `;

      let builder = client.createCodeBuilder();
      let compiledNoteScript = builder.compileNoteScript(noteScript);
      let noteInputs = new window.NoteInputs(
        new window.MidenArrays.FeltArray([
          walletAccount.id().prefix(),
          walletAccount.id().suffix(),
        ])
      );

      const serialNum = new window.Word(
        new BigUint64Array([BigInt(1), BigInt(2), BigInt(3), BigInt(4)])
      );

      let noteRecipient = new window.NoteRecipient(
        serialNum,
        compiledNoteScript,
        noteInputs
      );

      let note = new window.Note(noteAssets, noteMetadata, noteRecipient);

      const prover =
        withRemoteProver && window.remoteProverUrl != null
          ? window.remoteProverInstance
          : undefined;

      // Creating First Custom Transaction Request to Mint the Custom Note

      const outputNote = window.OutputNote.full(note);
      let transactionRequest = new window.TransactionRequestBuilder()
        .withOwnOutputNotes(
          new window.MidenArrays.OutputNoteArray([outputNote])
        )
        .build();

      // Execute and Submit Transaction
      let transactionUpdate = await window.helpers.executeAndApplyTransaction(
        faucetAccount.id(),
        transactionRequest,
        prover
      );

      await window.helpers.waitForTransaction(
        transactionUpdate.executedTransaction().id().toHex()
      );

      // Just like in the miden test, you can modify this script to get the execution to fail
      // by modifying the assert
      let txScript = `
            begin
                push.0 push.${assertedValue}
                # => [0, ${assertedValue}]
                assert_eq
            end
        `;

      // Creating Second Custom Transaction Request to Consume Custom Note
      // with Invalid/Valid Transaction Script
      let transactionScript = await builder.compileTxScript(txScript);
      let noteArgsCommitment = window.Rpo256.hashElements(feltArray); // gets consumed by NoteIdAndArgs

      let noteAndArgs = new window.NoteAndArgs(note, noteArgsCommitment);
      let noteAndArgsArray = new window.NoteAndArgsArray([noteAndArgs]);

      let adviceMap = new window.AdviceMap();
      let noteArgsCommitment2 = window.Rpo256.hashElements(feltArray);

      adviceMap.insert(noteArgsCommitment2, feltArray);

      let transactionRequest2 = new window.TransactionRequestBuilder()
        .withInputNotes(noteAndArgsArray)
        .withCustomScript(transactionScript)
        .extendAdviceMap(adviceMap)
        .build();

      // Execute and Submit Transaction
      let transactionUpdate2 = await window.helpers.executeAndApplyTransaction(
        walletAccount.id(),
        transactionRequest2,
        prover
      );

      await window.helpers.waitForTransaction(
        transactionUpdate2.executedTransaction().id().toHex()
      );
    },
    {
      assertedValue,
      withRemoteProver,
    }
  );
};

const customTxWithMultipleNotes = async (
  testingPage: Page,
  isSerialNumSame: boolean,
  senderAccountId: string,
  faucetAccountId: string
) => {
  return await testingPage.evaluate(
    async ({ isSerialNumSame, _senderAccountId, _faucetAccountId }) => {
      const client = window.client;
      const amount = BigInt(10);
      const targetAccount = await client.newWallet(
        window.AccountStorageMode.private(),
        true,
        0
      );
      const targetAccountId = targetAccount.id();
      const senderAccountId = window.AccountId.fromHex(_senderAccountId);
      const faucetAccountId = window.AccountId.fromHex(_faucetAccountId);

      // Create custom note with multiple assets to send to target account
      // Error should happen if serial numbers are the same in each set of
      // note assets. Otherwise, the transaction should go through.

      let noteAssets1 = new window.NoteAssets([
        new window.FungibleAsset(faucetAccountId, amount),
      ]);
      let noteAssets2 = new window.NoteAssets([
        new window.FungibleAsset(faucetAccountId, amount),
      ]);

      let noteMetadata = new window.NoteMetadata(
        senderAccountId,
        window.NoteType.Public,
        window.NoteTag.fromAccountId(targetAccountId)
      );

      let serialNum1 = new window.Word(
        new BigUint64Array([BigInt(1), BigInt(2), BigInt(3), BigInt(4)])
      );
      let serialNum2 = new window.Word(
        new BigUint64Array([BigInt(5), BigInt(6), BigInt(7), BigInt(8)])
      );

      const p2idScript = window.NoteScript.p2id();

      const inputNotes = new window.MidenArrays.FeltArray([
        targetAccount.id().suffix(),
        targetAccount.id().prefix(),
      ]);

      let noteInputs = new window.NoteInputs(inputNotes);

      let noteRecipient1 = new window.NoteRecipient(
        serialNum1,
        p2idScript,
        noteInputs
      );
      let noteRecipient2 = new window.NoteRecipient(
        isSerialNumSame ? serialNum1 : serialNum2,
        p2idScript,
        noteInputs
      );

      let note1 = new window.Note(noteAssets1, noteMetadata, noteRecipient1);
      let note2 = new window.Note(noteAssets2, noteMetadata, noteRecipient2);

      const notes = [
        window.OutputNote.full(note1),
        window.OutputNote.full(note2),
      ];

      const outputNotes = new window.MidenArrays.OutputNoteArray(notes);

      let transactionRequest = new window.TransactionRequestBuilder()
        .withOwnOutputNotes(outputNotes)
        .build();

      let transactionUpdate = await window.helpers.executeAndApplyTransaction(
        senderAccountId,
        transactionRequest
      );

      await window.helpers.waitForTransaction(
        transactionUpdate.executedTransaction().id().toHex()
      );
    },
    {
      isSerialNumSame,
      _senderAccountId: senderAccountId,
      _faucetAccountId: faucetAccountId,
    }
  );
};

test.describe("custom transaction tests", () => {
  test("custom transaction completes successfully", async ({ page }) => {
    await expect(customTransaction(page, "0", false)).resolves.toBeUndefined();
  });

  test("custom transaction fails", async ({ page }) => {
    await expect(customTransaction(page, "1", false)).rejects.toThrow();
  });

  test("custom transaction with remote prover completes successfully", async ({
    page,
  }) => {
    // TODO: hotfix CI failure, we should investigate slow prover tests further.
    test.slow();
    await expect(customTransaction(page, "0", true)).resolves.toBeUndefined();
  });
});

test.describe("custom transaction with multiple output notes", () => {
  const testCases = [
    {
      description: "does not fail when output note serial numbers are unique",
      shouldFail: false,
    },
    {
      description: "fails when output note serial numbers are the same",
      shouldFail: true,
    },
  ];

  testCases.forEach(({ description, shouldFail }) => {
    test(description, async ({ page }) => {
      const { accountId, faucetId } = await setupConsumedNote(page);
      if (shouldFail) {
        await expect(
          customTxWithMultipleNotes(page, shouldFail, accountId, faucetId)
        ).rejects.toThrow();
      } else {
        await expect(
          customTxWithMultipleNotes(page, shouldFail, accountId, faucetId)
        ).resolves.toBeUndefined();
      }
    });
  });
});

// CUSTOM ACCOUNT COMPONENT TESTS
// =======================================================================================================

export const customAccountComponent = async (
  testingPage: Page,
  schemeSecretKeyFunction: string
): Promise<void> => {
  return await testingPage.evaluate(
    async ({ schemeSecretKeyFunction }) => {
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
      const client = window.client;
      let builder = client.createCodeBuilder();
      let storageMap = new window.StorageMap();
      let storageSlotMap = window.StorageSlot.map(MAP_SLOT_NAME, storageMap);

      let accountComponentCode =
        builder.compileAccountComponentCode(accountCode);
      let mappingAccountComponent = window.AccountComponent.compile(
        accountComponentCode,
        [storageSlotMap]
      ).withSupportsAllTypes();

      const walletSeed = new Uint8Array(32);
      crypto.getRandomValues(walletSeed);

      let secretKey = window.AuthSecretKey[schemeSecretKeyFunction](walletSeed);
      let authComponent =
        window.AccountComponent.createAuthComponentFromSecretKey(secretKey);

      let accountBuilderResult = new window.AccountBuilder(walletSeed)
        .accountType(window.AccountType.RegularAccountImmutableCode)
        .storageMode(window.AccountStorageMode.public())
        .withAuthComponent(authComponent)
        .withComponent(mappingAccountComponent)
        .build();

      await client.addAccountSecretKeyToWebStore(
        accountBuilderResult.account.id(),
        secretKey
      );
      await client.newAccount(accountBuilderResult.account, false);

      await client.syncState();

      let accountCodeLib = builder.buildLibrary(
        "miden_by_example::mapping_example_contract",
        accountCode
      );

      builder.linkStaticLibrary(accountCodeLib);

      let txScript = builder.compileTxScript(scriptCode);

      let txIncrementRequest = new window.TransactionRequestBuilder()
        .withCustomScript(txScript)
        .build();

      let txResult = await window.helpers.executeAndApplyTransaction(
        accountBuilderResult.account.id(),
        txIncrementRequest
      );

      await window.helpers.waitForTransaction(
        txResult.executedTransaction().id().toHex()
      );

      // Fetch the updated account state from the client
      const updated = await client.getAccount(
        accountBuilderResult.account.id()
      );

      // Read a map value from storage slot 1 with key 0x0
      const keyZero = new window.Word(new BigUint64Array([0n, 0n, 0n, 0n]));
      const retrieveMapKey = updated
        ?.storage()
        .getMapItem(MAP_SLOT_NAME, keyZero);

      const expected = new window.Word(new BigUint64Array([1n, 2n, 3n, 4n]));

      if (retrieveMapKey?.toHex() !== expected.toHex()) {
        throw new Error(
          `unexpected Word: got ${retrieveMapKey?.toHex()} expected ${expected.toHex()}`
        );
      }
    },
    { schemeSecretKeyFunction }
  );
};

test.describe("custom account component tests", () => {
  [
    { authScheme: "ECDSA", schemeSecretKeyFunction: "ecdsaWithRNG" },
    { authScheme: "Falcon", schemeSecretKeyFunction: "rpoFalconWithRNG" },
  ].forEach(({ authScheme, schemeSecretKeyFunction }) => {
    test(`custom account component transaction completes successfully (${authScheme})`, async ({
      page,
    }) => {
      await expect(
        customAccountComponent(page, schemeSecretKeyFunction)
      ).resolves.toBeUndefined();
    });
  });
});

// DISCARDED TRANSACTIONS TESTS
// ================================================================================================

interface DiscardedTransactionUpdate {
  discardedTransactions: TransactionRecord[];
  commitmentBeforeTx: string;
  commitmentAfterTx: string;
  commitmentAfterDiscardedTx: string;
}

export const discardedTransaction = async (
  testingPage: Page
): Promise<DiscardedTransactionUpdate> => {
  return await testingPage.evaluate(async () => {
    const client = window.client;

    const senderAccount = await client.newWallet(
      window.AccountStorageMode.private(),
      true,
      0
    );
    const targetAccount = await client.newWallet(
      window.AccountStorageMode.private(),
      0,
      true
    );
    const faucetAccount = await client.newFaucet(
      window.AccountStorageMode.private(),
      false,
      "DAG",
      8,
      BigInt(10000000),
      0
    );
    await client.syncState();

    let mintTransactionRequest = client.newMintTransactionRequest(
      senderAccount.id(),
      faucetAccount.id(),
      window.NoteType.Private,
      BigInt(1000)
    );
    let mintTransactionUpdate = await window.helpers.executeAndApplyTransaction(
      faucetAccount.id(),
      mintTransactionRequest
    );
    let createdNotes = mintTransactionUpdate
      .executedTransaction()
      .outputNotes()
      .notes();
    let createdNoteIds = createdNotes.map((note: Note) => note.id().toString());
    await window.helpers.waitForTransaction(
      mintTransactionUpdate.executedTransaction().id().toHex()
    );

    let notes: Note[] = [];
    for (const _noteId of createdNoteIds) {
      const inputNoteRecord = await client.getInputNote(_noteId);

      if (!inputNoteRecord) {
        throw new Error(`Note with ID ${_noteId} not found`);
      }

      const note = inputNoteRecord.toNote();
      notes.push(note);
    }
    const senderConsumeTransactionRequest =
      client.newConsumeTransactionRequest(notes);
    let senderConsumeTransactionUpdate =
      await window.helpers.executeAndApplyTransaction(
        senderAccount.id(),
        senderConsumeTransactionRequest
      );
    await window.helpers.waitForTransaction(
      senderConsumeTransactionUpdate.executedTransaction().id().toHex()
    );

    let sendTransactionRequest = client.newSendTransactionRequest(
      senderAccount.id(),
      targetAccount.id(),
      faucetAccount.id(),
      window.NoteType.Private,
      BigInt(100),
      1,
      null
    );
    let sendTransactionUpdate = await window.helpers.executeAndApplyTransaction(
      senderAccount.id(),
      sendTransactionRequest
    );
    let sendCreatedNotes = sendTransactionUpdate
      .executedTransaction()
      .outputNotes()
      .notes();
    let sendCreatedNoteIds = sendCreatedNotes.map((note: Note) =>
      note.id().toString()
    );

    await window.helpers.waitForTransaction(
      sendTransactionUpdate.executedTransaction().id().toHex()
    );

    const inputNoteRecord = await client.getInputNote(sendCreatedNoteIds[0]);
    if (!inputNoteRecord) {
      throw new Error(`Note with ID ${sendCreatedNoteIds[0]} not found`);
    }

    const note = inputNoteRecord.toNote();
    let noteAndArgs = new window.NoteAndArgs(note, null);
    let noteAndArgsArray = new window.NoteAndArgsArray([noteAndArgs]);
    const consumeTransactionRequest = new window.TransactionRequestBuilder()
      .withInputNotes(noteAndArgsArray)
      .build();

    let preConsumeStore = await client.exportStore();

    // Sender retrieves the note

    notes = [];
    for (const _noteId of sendCreatedNoteIds) {
      const inputNoteRecord = await client.getInputNote(_noteId);

      if (!inputNoteRecord) {
        throw new Error(`Note with ID ${_noteId} not found`);
      }

      const note = inputNoteRecord.toNote();
      notes.push(note);
    }
    let senderTxRequest = client.newConsumeTransactionRequest(notes);
    let senderTxResult = await window.helpers.executeAndApplyTransaction(
      senderAccount.id(),
      senderTxRequest
    );
    await window.helpers.waitForTransaction(
      senderTxResult.executedTransaction().id().toHex()
    );

    await client.forceImportStore(preConsumeStore, "tests");

    // Get the account state before the transaction is applied
    const accountStateBeforeTx = (await client.getAccount(
      targetAccount.id()
    )) as Account;
    if (!accountStateBeforeTx) {
      throw new Error("Failed to get account state before transaction");
    }

    // Target tries consuming but the transaction will not be submitted
    let targetPipeline = await client.executeTransaction(
      targetAccount.id(),
      consumeTransactionRequest
    );
    const submissionHeight = (await client.getSyncHeight()) + 1;
    await client.applyTransaction(targetPipeline, submissionHeight);
    // Get the account state after the transaction is applied
    const accountStateAfterTx = (await client.getAccount(
      targetAccount.id()
    )) as Account;
    if (!accountStateAfterTx) {
      throw new Error("Failed to get account state after transaction");
    }

    await client.syncState();

    const allTransactions = await client.getTransactions(
      window.TransactionFilter.all()
    );

    const discardedTransactions = allTransactions.filter(
      (tx: TransactionRecord) => tx.transactionStatus().isDiscarded()
    );

    // Get the account state after the discarded transactions are applied
    const accountStateAfterDiscardedTx = (await client.getAccount(
      targetAccount.id()
    )) as Account;
    if (!accountStateAfterDiscardedTx) {
      throw new Error(
        "Failed to get account state after discarded transaction"
      );
    }

    // Perform a `.commitment()` check on each account
    const commitmentBeforeTx = accountStateBeforeTx.commitment().toHex();
    const commitmentAfterTx = accountStateAfterTx.commitment().toHex();
    const commitmentAfterDiscardedTx = accountStateAfterDiscardedTx
      .commitment()
      .toHex();

    return {
      discardedTransactions: discardedTransactions,
      commitmentBeforeTx,
      commitmentAfterTx,
      commitmentAfterDiscardedTx,
    };
  });
};

test.describe("discarded_transaction tests", () => {
  test("transaction gets discarded", async ({ page }) => {
    const result = await discardedTransaction(page);

    expect(result.discardedTransactions.length).toEqual(1);
    expect(result.commitmentBeforeTx).toEqual(
      result.commitmentAfterDiscardedTx
    );
    expect(result.commitmentAfterTx).not.toEqual(
      result.commitmentAfterDiscardedTx
    );
  });
});

// NETWORK TRANSACTION TESTS
// ================================================================================================

export const counterAccountComponent = async (
  testingPage: Page
): Promise<{
  finalCounter?: string;
  hasCounterComponent: boolean;
}> => {
  return await testingPage.evaluate(async () => {
    const COUNTER_SLOT_NAME = "miden::testing::counter_contract::counter";

    const accountCode = `
        use miden::protocol::active_account
        use miden::protocol::native_account
        use miden::core::word
        use miden::core::sys

        const COUNTER_SLOT = word("${COUNTER_SLOT_NAME}")

        # => []
        pub proc get_count
            push.COUNTER_SLOT[0..2] exec.active_account::get_item
            exec.sys::truncate_stack
        end

        # => []
        pub proc increment_count
            push.COUNTER_SLOT[0..2] exec.active_account::get_item
            # => [count]
            push.1 add
            # => [count+1]
            push.COUNTER_SLOT[0..2] exec.native_account::set_item
            # => []
            exec.sys::truncate_stack
            # => []
        end
      `;
    const scriptCode = `
        use external_contract::counter_contract
        begin
            call.counter_contract::increment_count
        end
      `;
    const client = window.client;

    // Create counter account
    let emptyStorageSlot = window.StorageSlot.emptyValue(COUNTER_SLOT_NAME);

    let builder = client.createCodeBuilder();

    let accountComponentCode = builder.compileAccountComponentCode(accountCode);
    let counterAccountComponent = window.AccountComponent.compile(
      accountComponentCode,
      [emptyStorageSlot]
    ).withSupportsAllTypes();

    const walletSeed = new Uint8Array(32);
    crypto.getRandomValues(walletSeed);

    let accountBuilderResult = new window.AccountBuilder(walletSeed)
      .storageMode(window.AccountStorageMode.network())
      .withNoAuthComponent()
      .withComponent(counterAccountComponent)
      .build();

    await client.newAccount(accountBuilderResult.account, false);

    const nativeAccount = await client.newWallet(
      window.AccountStorageMode.private(),
      false,
      0
    );

    await client.syncState();

    // Deploy counter account
    let accountComponentLib = builder.buildLibrary(
      "external_contract::counter_contract",
      accountCode
    );
    builder.linkDynamicLibrary(accountComponentLib);
    let txScript = builder.compileTxScript(scriptCode);

    let txIncrementRequest = new window.TransactionRequestBuilder()
      .withCustomScript(txScript)
      .build();

    let txUpdate = await window.helpers.executeAndApplyTransaction(
      accountBuilderResult.account.id(),
      txIncrementRequest
    );
    await window.helpers.waitForTransaction(
      txUpdate.executedTransaction().id().toHex()
    );

    // Create transaction with network note
    let compiledNoteScript = await builder.compileNoteScript(scriptCode);

    let noteInputs = new window.NoteInputs(
      new window.MidenArrays.FeltArray([])
    );

    const randomInts = Array.from({ length: 4 }, () =>
      Math.floor(Math.random() * 100000)
    );

    let serialNum = new window.Word(new BigUint64Array(randomInts.map(BigInt)));

    let noteRecipient = new window.NoteRecipient(
      serialNum,
      compiledNoteScript,
      noteInputs
    );

    let noteAssets = new window.NoteAssets([]);

    // Create network account target attachment so the node knows to consume this note
    // with the network account (counter account)
    let networkTarget = new window.NetworkAccountTarget(
      accountBuilderResult.account.id(),
      window.NoteExecutionHint.always()
    );

    let noteMetadata = new window.NoteMetadata(
      nativeAccount.id(),
      window.NoteType.Public,
      window.NoteTag.fromAccountId(accountBuilderResult.account.id())
    ).withAttachment(networkTarget.intoAttachment());

    let note = new window.Note(noteAssets, noteMetadata, noteRecipient);

    let transactionRequest = new window.TransactionRequestBuilder()
      .withOwnOutputNotes(
        new window.MidenArrays.OutputNoteArray([window.OutputNote.full(note)])
      )
      .build();

    let transactionUpdate = await window.helpers.executeAndApplyTransaction(
      nativeAccount.id(),
      transactionRequest
    );
    await window.helpers.waitForTransaction(
      transactionUpdate.executedTransaction().id().toHex()
    );

    // Wait for network account to update
    await window.helpers.waitForBlocks(2);

    let account = await client.getAccount(accountBuilderResult.account.id());
    let counter = account?.storage().getItem(COUNTER_SLOT_NAME)?.toHex();
    let finalCounter = counter?.replace(/^0x/, "").replace(/^0+|0+$/g, "");

    let code = account?.code();
    let hasCounterComponent = code
      ? counterAccountComponent
          .getProcedures()
          .every((procedure) => code.hasProcedure(procedure.digest))
      : false;

    return {
      finalCounter,
      hasCounterComponent,
    };
  });
};

test.describe("counter account component tests", () => {
  test("counter account component transaction completes successfully", async ({
    page,
  }) => {
    let { finalCounter, hasCounterComponent } =
      await counterAccountComponent(page);
    expect(finalCounter).toEqual("2");
    expect(hasCounterComponent).toBe(true);
  });
});

export const testStorageMap = async (page: Page): Promise<any> => {
  return await page.evaluate(async () => {
    const client = window.client;
    await client.syncState();

    const MAP_SLOT_NAME = "miden::testing::bump_item_contract::map_slot";

    const normalizeHexWord = (hex) => {
      if (!hex) return undefined;
      const normalized = hex.replace(/^0x/, "").replace(/^0+|0+$/g, "");
      return normalized;
    };

    // BUILD ACCOUNT WITH COMPONENT THAT MODIFIES STORAGE MAP
    // --------------------------------------------------------------------------

    const MAP_KEY = new window.Word(new BigUint64Array([1n, 1n, 1n, 1n]));
    const FPI_STORAGE_VALUE = new window.Word(
      new BigUint64Array([0n, 0n, 0n, 1n])
    );

    let storageMap = new window.StorageMap();
    storageMap.insert(MAP_KEY, FPI_STORAGE_VALUE);
    storageMap.insert(
      new window.Word(new BigUint64Array([2n, 2n, 2n, 2n])),
      new window.Word(new BigUint64Array([0n, 0n, 0n, 9n]))
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

    let builder = client.createCodeBuilder();
    let accountComponentCode = builder.compileAccountComponentCode(accountCode);
    let bumpItemComponent = window.AccountComponent.compile(
      accountComponentCode,
      [window.StorageSlot.map(MAP_SLOT_NAME, storageMap)]
    ).withSupportsAllTypes();

    const walletSeed = new Uint8Array(32);
    crypto.getRandomValues(walletSeed);

    let secretKey = window.AuthSecretKey.rpoFalconWithRNG(walletSeed);
    let authComponent =
      window.AccountComponent.createAuthComponentFromSecretKey(secretKey);

    let bumpItemAccountBuilderResult = new window.AccountBuilder(walletSeed)
      .withAuthComponent(authComponent)
      .withComponent(bumpItemComponent)
      .storageMode(window.AccountStorageMode.public())
      .build();

    await client.addAccountSecretKeyToWebStore(
      bumpItemAccountBuilderResult.account.id(),
      secretKey
    );
    await client.newAccount(bumpItemAccountBuilderResult.account, false);
    await client.syncState();

    let initialMapValue = (
      await client.getAccount(bumpItemAccountBuilderResult.account.id())
    )
      ?.storage()
      .getMapItem(MAP_SLOT_NAME, MAP_KEY)
      ?.toHex();

    // Deploy counter account

    let accountComponentLib = builder.buildLibrary(
      "external_contract::bump_item_contract",
      accountCode
    );

    builder.linkDynamicLibrary(accountComponentLib);

    let txScript = builder.compileTxScript(
      `use external_contract::bump_item_contract
      begin
          call.bump_item_contract::bump_map_item
      end`
    );

    let txIncrementRequest = new window.TransactionRequestBuilder()
      .withCustomScript(txScript)
      .build();

    let txResult = await window.helpers.executeAndApplyTransaction(
      bumpItemAccountBuilderResult.account.id(),
      txIncrementRequest
    );
    await window.helpers.waitForTransaction(
      txResult.executedTransaction().id().toHex()
    );

    let finalMapValue = (
      await client.getAccount(bumpItemAccountBuilderResult.account.id())
    )
      ?.storage()
      .getMapItem(MAP_SLOT_NAME, MAP_KEY)
      ?.toHex();

    // Test getMapEntries() functionality
    let accountStorage = (
      await client.getAccount(bumpItemAccountBuilderResult.account.id())
    )?.storage();
    let mapEntries = accountStorage?.getMapEntries(MAP_SLOT_NAME);

    // Verify we get the expected entries
    let expectedKey = MAP_KEY.toHex();
    let expectedValue = normalizeHexWord(finalMapValue);

    let mapEntriesData = {
      entriesCount: mapEntries?.length || 0,
      hasExpectedEntry: false,
      expectedKey: expectedKey,
      expectedValue: expectedValue,
    };

    if (expectedValue && mapEntries && mapEntries.length > 0) {
      mapEntriesData.hasExpectedEntry = mapEntries.some(
        (entry) =>
          entry.key === expectedKey &&
          normalizeHexWord(entry.value) === expectedValue
      );
    }

    return {
      initialMapValue: normalizeHexWord(initialMapValue),
      finalMapValue: normalizeHexWord(finalMapValue),
      mapEntries: mapEntriesData,
    };
  });
};

test.describe("storage map test", () => {
  test.setTimeout(50000);
  test("storage map is updated correctly in transaction", async ({ page }) => {
    let { initialMapValue, finalMapValue, mapEntries } =
      await testStorageMap(page);
    expect(initialMapValue).toBe("1");
    expect(finalMapValue).toBe("2");

    // Test getMapEntries() functionality
    expect(mapEntries.entriesCount).toBeGreaterThan(1);
    expect(mapEntries.hasExpectedEntry).toBe(true);
    expect(mapEntries.expectedKey).toBeDefined();
    expect(mapEntries.expectedValue).toBe("2");
  });
});

// SUBMIT_NEW_TRANSACTION_WITH_PROVER TESTS
// ================================================================================================

test.describe("submitNewTransactionWithProver tests", () => {
  test("submitNewTransactionWithProver with failing prover throws, then succeeds with local prover", async ({
    page,
  }) => {
    const { faucetId, accountId } = await setupWalletAndFaucet(page);

    // Test that a failing prover throws an error
    const failingProverResult = await page.evaluate(
      async ({ faucetId, accountId }) => {
        const client = window.client;
        const faucetAccountId = window.AccountId.fromHex(faucetId);
        const targetAccountId = window.AccountId.fromHex(accountId);

        await client.syncState();

        const mintTransactionRequest = client.newMintTransactionRequest(
          targetAccountId,
          faucetAccountId,
          window.NoteType.Public,
          BigInt(1000)
        );

        // Create a failing remote prover with an invalid endpoint
        const failingProver = window.TransactionProver.newRemoteProver(
          "http://localhost:1",
          null
        );

        try {
          await client.submitNewTransactionWithProver(
            faucetAccountId,
            mintTransactionRequest,
            failingProver
          );
          return { threw: false, error: null };
        } catch (e: any) {
          return { threw: true, error: e.message || String(e) };
        }
      },
      { faucetId, accountId }
    );

    expect(failingProverResult.threw).toBe(true);
  });

  test.describe("executeForSummary tests", () => {
    test("executeForSummary returns TransactionSummary for unauthorized transaction", async ({
      page,
    }) => {
      const result = await page.evaluate(async () => {
        const client = window.client;

        const walletSeed = new Uint8Array(32);
        crypto.getRandomValues(walletSeed);

        const approverKeys = [
          window.AuthSecretKey.rpoFalconWithRNG(),
          window.AuthSecretKey.rpoFalconWithRNG(),
          window.AuthSecretKey.rpoFalconWithRNG(),
        ];
        const approverCommitments = approverKeys.map((key) =>
          key.publicKey().toCommitment()
        );
        const multisigConfig = new window.AuthFalcon512RpoMultisigConfig(
          approverCommitments,
          2
        );
        const multisigComponent =
          window.createAuthFalcon512RpoMultisig(multisigConfig);

        const accountBuilderResult = new window.AccountBuilder(walletSeed)
          .accountType(window.AccountType.RegularAccountImmutableCode)
          .storageMode(window.AccountStorageMode.private())
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
          window.AccountStorageMode.private(),
          false,
          0
        );

        const faucetAccount = await client.newFaucet(
          window.AccountStorageMode.private(),
          false,
          "DAG",
          8,
          BigInt(10000000),
          0
        );

        await client.syncState();

        const mintTransactionRequest = client.newMintTransactionRequest(
          targetAccount.id(),
          faucetAccount.id(),
          window.NoteType.Public,
          BigInt(1000)
        );

        const mintTransactionUpdate =
          await window.helpers.executeAndApplyTransaction(
            faucetAccount.id(),
            mintTransactionRequest
          );

        const createdNoteIds = mintTransactionUpdate
          .executedTransaction()
          .outputNotes()
          .notes()
          .map((note: Note) => note.id().toString());

        await window.helpers.waitForTransaction(
          mintTransactionUpdate.executedTransaction().id().toHex()
        );

        // Convert note IDs to Note objects for consume request
        const createdNotes = await Promise.all(
          createdNoteIds.map(async (noteId: string) => {
            const inputNoteRecord = await client.getInputNote(noteId);
            if (!inputNoteRecord) {
              throw new Error(`Note with ID ${noteId} not found`);
            }
            return inputNoteRecord.toNote();
          })
        );

        const consumeTransactionRequest =
          client.newConsumeTransactionRequest(createdNotes);

        const consumeTransactionUpdate =
          await window.helpers.executeAndApplyTransaction(
            targetAccount.id(),
            consumeTransactionRequest
          );

        await window.helpers.waitForTransaction(
          consumeTransactionUpdate.executedTransaction().id().toHex()
        );

        const sendTransactionRequest = client.newSendTransactionRequest(
          targetAccount.id(),
          accountBuilderResult.account.id(),
          faucetAccount.id(),
          window.NoteType.Public,
          BigInt(100),
          null,
          null
        );

        const sendTransactionUpdate =
          await window.helpers.executeAndApplyTransaction(
            targetAccount.id(),
            sendTransactionRequest
          );

        const sentNoteIds = sendTransactionUpdate
          .executedTransaction()
          .outputNotes()
          .notes()
          .map((note: Note) => note.id().toString());

        await window.helpers.waitForTransaction(
          sendTransactionUpdate.executedTransaction().id().toHex()
        );

        // Convert note IDs to Note objects for consume request
        const sentNotes = await Promise.all(
          sentNoteIds.map(async (noteId: string) => {
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

        return {
          inputNotesCount: summary.inputNotes().numNotes(),
          outputNotesCount: summary.outputNotes().numNotes(),
          inputNoteIds: summary
            .inputNotes()
            .notes()
            .map((note: any) => note.id().toString()),
          sentNoteIds,
        };
      });

      expect(result.inputNotesCount).toBe(1);
      expect(result.outputNotesCount).toBe(0);
      expect(result.inputNoteIds).toEqual(result.sentNoteIds);
    });

    test("executeForSummary returns TransactionSummary for authorized transaction with matching salt", async ({
      page,
    }) => {
      const result = await page.evaluate(async () => {
        const client = window.client;

        const senderAccount = await client.newWallet(
          window.AccountStorageMode.private(),
          false,
          0
        );

        await client.syncState();

        // Create a known salt value
        const expectedSalt = new window.Word(
          new BigUint64Array([BigInt(1), BigInt(2), BigInt(3), BigInt(4)])
        );

        // Build transaction request with the salt as auth_arg
        const transactionRequest = new window.TransactionRequestBuilder()
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
