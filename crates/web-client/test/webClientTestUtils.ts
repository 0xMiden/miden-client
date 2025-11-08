// @ts-nocheck
import { expect } from "chai";
import { TransactionProver, WebClient } from "../dist";
import test from "./playwright.global.setup";
import { Page } from "@playwright/test";

interface MintTransactionUpdate {
  transactionId: string;
  numOutputNotesCreated: number;
  nonce: string | undefined;
  createdNoteId: string;
}

export enum StorageMode {
  PRIVATE = "private",
  PUBLIC = "public",
}

// SDK functions

export const mintTransaction = async (
  testingPage: Page,
  targetAccountId: string,
  faucetAccountId: string,
  withRemoteProver: boolean = false,
  sync: boolean = true,
  publicNote: boolean = false
): Promise<MintTransactionResult> => {
  return await testingPage.evaluate(
    async ({
      _targetAccountId,
      _faucetAccountId,
      _withRemoteProver,
      _sync,
      _publicNote,
    }: {
      _targetAccountId: string;
      _faucetAccountId: string;
      _withRemoteProver: boolean;
      _sync: boolean;
      _publicNote: boolean;
    }) => {
      const client = window.client;
      await client.syncState();

      const targetAccountId = window.AccountId.fromHex(_targetAccountId);
      const faucetAccountId = window.AccountId.fromHex(_faucetAccountId);

      const mintTransactionRequest = client.newMintTransactionRequest(
        targetAccountId,
        faucetAccountId,
        _publicNote ? window.NoteType.Public : window.NoteType.Private,
        BigInt(1000)
      );
      const prover =
        _withRemoteProver && window.remoteProverUrl != null
          ? window.remoteProverInstance
          : undefined;

      const mintTransactionResult =
        await window.helpers.executeAndApplyTransaction(
          faucetAccountId,
          mintTransactionRequest,
          prover
        );

      if (_sync) {
        await window.helpers.waitForTransaction(
          mintTransactionResult.executedTransaction().id().toHex()
        );
      }

      return {
        transactionId: mintTransactionResult.executedTransaction().id().toHex(),
        numOutputNotesCreated: mintTransactionResult.createdNotes().numNotes(),
        nonce: mintTransactionResult.accountDelta().nonceDelta().toString(),
        createdNoteId: mintTransactionResult
          .createdNotes()
          .notes()[0]
          .id()
          .toString(),
      };
    },
    {
      _targetAccountId: targetAccountId,
      _faucetAccountId: faucetAccountId,
      _withRemoteProver: withRemoteProver,
      _sync: sync,
      _publicNote: publicNote,
    }
  );
};

export const mintPublicTransaction = async (
  testingPage: Page,
  targetAccountId: string,
  faucetAccountId: string,
  withRemoteProver: boolean = false,
  sync: boolean = true
): Promise<MintTransactionUpdate> => {
  return await testingPage.evaluate(
    async ({
      _targetAccountId,
      _faucetAccountId,
      _withRemoteProver,
      _sync,
    }: {
      _targetAccountId: string;
      _faucetAccountId: string;
      _withRemoteProver: boolean;
      _sync: boolean;
    }) => {
      const client = window.client;
      await client.syncState();

      const targetAccountId = window.AccountId.fromHex(_targetAccountId);
      const faucetAccountId = window.AccountId.fromHex(_faucetAccountId);

      const mintTransactionRequest = client.newMintTransactionRequest(
        targetAccountId,
        faucetAccountId,
        window.NoteType.Public,
        BigInt(1000)
      );
      const prover =
        _withRemoteProver && window.remoteProverUrl != null
          ? window.remoteProverInstance
          : undefined;
      const mintTransactionUpdate =
        await window.helpers.executeAndApplyTransaction(
          faucetAccountId,
          mintTransactionRequest,
          prover
        );

      if (_sync) {
        await window.helpers.waitForTransaction(
          mintTransactionUpdate.executedTransaction().id().toHex()
        );
      }

      return {
        transactionId: mintTransactionUpdate.executedTransaction().id().toHex(),
        numOutputNotesCreated: mintTransactionUpdate
          .executedTransaction()
          .outputNotes()
          .numNotes(),
        nonce: mintTransactionUpdate
          .executedTransaction()
          .accountDelta()
          .nonceDelta()
          .toString(),
        createdNoteId: mintTransactionUpdate
          .executedTransaction()
          .outputNotes()
          .notes()[0]
          .id()
          .toString(),
      };
    },
    {
      _targetAccountId: targetAccountId,
      _faucetAccountId: faucetAccountId,
      _withRemoteProver: withRemoteProver,
      _sync: sync,
    }
  );
};

export const getSyncHeight = async (testingPage: Page) => {
  return await testingPage.evaluate(async () => {
    const client = window.client;
    let summary = await client.syncState();
    return summary.blockNum();
  });
};

export const sendTransaction = async (
  testingPage: Page,
  senderAccountId: string,
  targetAccountId: string,
  faucetAccountId: string,
  recallHeight?: number,
  withRemoteProver: boolean = false
) => {
  return testingPage.evaluate(
    async ({
      _senderAccountId,
      _targetAccountId,
      _faucetAccountId,
      _recallHeight,
      _withRemoteProver,
    }: {
      _senderAccountId: string;
      _targetAccountId: string;
      _faucetAccountId: string;
      _recallHeight?: number;
      _withRemoteProver: boolean;
    }): Promise<string[]> => {
      const client = window.client;

      await client.syncState();

      const senderAccountId = window.AccountId.fromHex(_senderAccountId);
      const targetAccountId = window.AccountId.fromHex(_targetAccountId);
      const faucetAccountId = window.AccountId.fromHex(_faucetAccountId);

      let mintTransactionRequest = client.newMintTransactionRequest(
        senderAccountId,
        faucetAccountId,
        window.NoteType.Private,
        BigInt(1000)
      );

      const prover =
        _withRemoteProver && window.remoteProverUrl != null
          ? window.remoteProverInstance
          : undefined;

      let mintTransactionUpdate =
        await window.helpers.executeAndApplyTransaction(
          faucetAccountId,
          mintTransactionRequest,
          prover
        );

      let createdNote = mintTransactionUpdate
        .executedTransaction()
        .outputNotes()
        .notes()[0]
        .intoFull();

      if (!createdNote) {
        throw new Error("Created note is undefined");
      }

      let noteAndArgs = new window.NoteAndArgs(createdNote, null);
      let noteAndArgsArray = new window.NoteAndArgsArray([noteAndArgs]);

      let txRequest = new window.TransactionRequestBuilder()
        .withUnauthenticatedInputNotes(noteAndArgsArray)
        .build();

      let consumeTransactionUpdate =
        await window.helpers.executeAndApplyTransaction(
          senderAccountId,
          txRequest,
          prover
        );

      let sendTransactionRequest = client.newSendTransactionRequest(
        senderAccountId,
        targetAccountId,
        faucetAccountId,
        window.NoteType.Public,
        BigInt(100),
        _recallHeight,
        null
      );
      let sendTransactionUpdate =
        await window.helpers.executeAndApplyTransaction(
          senderAccountId,
          sendTransactionRequest,
          prover
        );
      let sendCreatedNotes = sendTransactionUpdate
        .executedTransaction()
        .outputNotes()
        .notes();
      let sendCreatedNoteIds = sendCreatedNotes.map((note) =>
        note.id().toString()
      );

      await window.helpers.waitForTransaction(
        sendTransactionUpdate.executedTransaction().id().toHex()
      );

      return sendCreatedNoteIds;
    },
    {
      _senderAccountId: senderAccountId,
      _targetAccountId: targetAccountId,
      _faucetAccountId: faucetAccountId,
      _recallHeight: recallHeight,
      _withRemoteProver: withRemoteProver,
    }
  );
};

export interface SwapTransactionUpdate {
  accountAAssets: { assetId: string; amount: string }[] | undefined;
  accountBAssets: { assetId: string; amount: string }[] | undefined;
}

export const swapTransaction = async (
  testingPage: Page,
  accountAId: string,
  accountBId: string,
  assetAFaucetId: string,
  assetAAmount: bigint,
  assetBFaucetId: string,
  assetBAmount: bigint,
  swapNoteType: string = "private",
  paybackNoteType: string = "private",
  withRemoteProver: boolean = false
): Promise<SwapTransactionUpdate> => {
  return await testingPage.evaluate(
    async ({
      _accountAId,
      _accountBId,
      _assetAFaucetId,
      _assetAAmount,
      _assetBFaucetId,
      _assetBAmount,
      _swapNoteType,
      _paybackNoteType,
      _withRemoteProver,
    }: {
      _accountAId: string;
      _accountBId: string;
      _assetAFaucetId: string;
      _assetAAmount: bigint;
      _assetBFaucetId: string;
      _assetBAmount: bigint;
      _swapNoteType: string;
      _paybackNoteType: string;
      _withRemoteProver: boolean;
    }) => {
      const client = window.client;

      await client.syncState();

      const accountAId = window.AccountId.fromHex(_accountAId);
      const accountBId = window.AccountId.fromHex(_accountBId);
      const assetAFaucetId = window.AccountId.fromHex(_assetAFaucetId);
      const assetBFaucetId = window.AccountId.fromHex(_assetBFaucetId);

      const swapNoteType =
        _swapNoteType === "public"
          ? window.NoteType.Public
          : window.NoteType.Private;
      const paybackNoteType =
        _paybackNoteType === "public"
          ? window.NoteType.Public
          : window.NoteType.Private;

      const prover =
        _withRemoteProver && window.remoteProverUrl != null
          ? window.remoteProverInstance
          : undefined;

      // Swap transaction

      let swapTransactionRequest = client.newSwapTransactionRequest(
        accountAId,
        assetAFaucetId,
        _assetAAmount,
        assetBFaucetId,
        _assetBAmount,
        swapNoteType,
        paybackNoteType
      );

      let expectedOutputNotes = swapTransactionRequest.expectedOutputOwnNotes();
      let expectedPaybackNoteDetails = swapTransactionRequest
        .expectedFutureNotes()
        .map((futureNote) => futureNote.noteDetails);

      let swapTransactionUpdate =
        await window.helpers.executeAndApplyTransaction(
          accountAId,
          swapTransactionRequest,
          prover
        );

      await window.helpers.waitForTransaction(
        swapTransactionUpdate.executedTransaction().id().toHex()
      );

      // Consuming swap note for account B

      let txRequest1 = client.newConsumeTransactionRequest([
        expectedOutputNotes[0].id().toString(),
      ]);

      let consumeTransaction1Result =
        await window.helpers.executeAndApplyTransaction(
          accountBId,
          txRequest1,
          prover
        );

      await window.helpers.waitForTransaction(
        consumeTransaction1Result.executedTransaction().id().toHex()
      );

      // Consuming payback note for account A

      let txRequest2 = client.newConsumeTransactionRequest([
        expectedPaybackNoteDetails[0].id().toString(),
      ]);

      let consumeTransaction2Result =
        await window.helpers.executeAndApplyTransaction(
          accountAId,
          txRequest2,
          prover
        );

      await window.helpers.waitForTransaction(
        consumeTransaction2Result.executedTransaction().id().toHex()
      );

      // Fetching assets from both accounts after the swap

      let accountA = await client.getAccount(accountAId);
      let accountAAssets = accountA
        ?.vault()
        .fungibleAssets()
        .map((asset) => {
          return {
            assetId: asset.faucetId().toString(),
            amount: asset.amount().toString(),
          };
        });

      let accountB = await client.getAccount(accountBId);
      let accountBAssets = accountB
        ?.vault()
        .fungibleAssets()
        .map((asset) => {
          return {
            assetId: asset.faucetId().toString(),
            amount: asset.amount().toString(),
          };
        });

      return {
        accountAAssets,
        accountBAssets,
      };
    },
    {
      _accountAId: accountAId,
      _accountBId: accountBId,
      _assetAFaucetId: assetAFaucetId,
      _assetAAmount: assetAAmount,
      _assetBFaucetId: assetBFaucetId,
      _assetBAmount: assetBAmount,
      _swapNoteType: swapNoteType,
      _paybackNoteType: paybackNoteType,
      _withRemoteProver: withRemoteProver,
    }
  );
};

export interface NewAccountTestResult {
  id: string;
  nonce: string;
  vaultCommitment: string;
  storageCommitment: string;
  codeCommitment: string;
  isFaucet: boolean;
  isRegularAccount: boolean;
  isUpdatable: boolean;
  isPublic: boolean;
  isNew: boolean;
}
interface createNewWalletParams {
  storageMode: StorageMode;
  mutable: boolean;
  clientSeed?: Uint8Array;
  isolatedClient?: boolean;
  walletSeed?: Uint8Array;
  serializeWalletSeed?: string;
  serializeClientSeed?: string;
}
export const createNewWallet = async (
  testingPage: Page,
  {
    storageMode,
    mutable,
    clientSeed,
    isolatedClient,
    walletSeed,
  }: createNewWalletParams
): Promise<NewAccountTestResult> => {
  // Serialize initSeed for Puppeteer
  const serializedClientSeed = clientSeed ? Array.from(clientSeed) : undefined;
  const serializedWalletSeed = walletSeed ? Array.from(walletSeed) : undefined;

  return await testingPage.evaluate(
    async ({
      storageMode,
      mutable,
      _serializedWalletSeed,
      _serializedClientSeed,
      isolatedClient,
    }: createNewWalletParams) => {
      if (isolatedClient) {
        // Reconstruct Uint8Array inside the browser context
        const _clientSeed = _serializedClientSeed
          ? new Uint8Array(_serializedClientSeed)
          : undefined;

        await window.helpers.refreshClient(_clientSeed);
      }

      let _walletSeed;
      if (_serializedWalletSeed) {
        _walletSeed = new Uint8Array(_serializedWalletSeed);
      }

      let client = window.client;
      const accountStorageMode =
        window.AccountStorageMode.tryFromStr(storageMode);

      const newWallet = await client.newWallet(
        accountStorageMode,
        mutable,
        _walletSeed
      );

      return {
        id: newWallet.id().toString(),
        nonce: newWallet.nonce().toString(),
        vaultCommitment: newWallet.vault().root().toHex(),
        storageCommitment: newWallet.storage().commitment().toHex(),
        codeCommitment: newWallet.code().commitment().toHex(),
        isFaucet: newWallet.isFaucet(),
        isRegularAccount: newWallet.isRegularAccount(),
        isUpdatable: newWallet.isUpdatable(),
        isPublic: newWallet.isPublic(),
        isNew: newWallet.isNew(),
      };
    },
    {
      storageMode: storageMode,
      mutable: mutable,
      _serializedClientSeed: serializedClientSeed,
      isolatedClient: isolatedClient,
      _serializedWalletSeed: serializedWalletSeed,
    }
  );
};

export const createNewFaucet = async (
  testingPage: Page,
  storageMode: StorageMode = StorageMode.PUBLIC,
  nonFungible: boolean = false,
  tokenSymbol: string = "DAG",
  decimals: number = 8,
  maxSupply: bigint = BigInt(10000000)
): Promise<NewAccountTestResult> => {
  return await testingPage.evaluate(
    async ({ storageMode, nonFungible, tokenSymbol, decimals, maxSupply }) => {
      const client = window.client;
      const accountStorageMode =
        window.AccountStorageMode.tryFromStr(storageMode);
      const newFaucet = await client.newFaucet(
        accountStorageMode,
        nonFungible,
        tokenSymbol,
        decimals,
        maxSupply
      );
      return {
        id: newFaucet.id().toString(),
        nonce: newFaucet.nonce().toString(),
        vaultCommitment: newFaucet.vault().root().toHex(),
        storageCommitment: newFaucet.storage().commitment().toHex(),
        codeCommitment: newFaucet.code().commitment().toHex(),
        isFaucet: newFaucet.isFaucet(),
        isRegularAccount: newFaucet.isRegularAccount(),
        isUpdatable: newFaucet.isUpdatable(),
        isPublic: newFaucet.isPublic(),
        isNew: newFaucet.isNew(),
      };
    },
    {
      storageMode,
      nonFungible,
      tokenSymbol,
      decimals,
      maxSupply,
    }
  );
};

export const fundAccountFromFaucet = async (
  page: Page,
  accountId: string,
  faucetId: string
) => {
  const mintResult = await mintTransaction(page, accountId, faucetId);
  return await consumeTransaction(
    page,
    accountId,
    faucetId,
    mintResult.createdNoteId
  );
};

export const getAccountBalance = async (
  testingPage: Page,
  accountId: string,
  faucetId: string
) => {
  return await testingPage.evaluate(
    async ({ accountId, faucetId }) => {
      const client = window.client;
      const account = await client.getAccount(
        window.AccountId.fromHex(accountId)
      );
      let balance = BigInt(0);
      if (account) {
        balance = account
          .vault()
          .getBalance(window.AccountId.fromHex(faucetId));
      }
      return balance;
    },
    {
      accountId,
      faucetId,
    }
  );
};

interface ConsumeTransactionUpdate {
  transactionId: string;
  nonce: string | undefined;
  numConsumedNotes: number;
  targetAccountBalance: string;
}

export const consumeTransaction = async (
  testingPage: Page,
  targetAccountId: string,
  faucetId: string,
  noteId: string,
  withRemoteProver: boolean = false
): Promise<ConsumeTransactionUpdate> => {
  return await testingPage.evaluate(
    async ({ _targetAccountId, _faucetId, _noteId, _withRemoteProver }) => {
      const client = window.client;

      await client.syncState();

      const targetAccountId = window.AccountId.fromHex(_targetAccountId);
      const faucetId = window.AccountId.fromHex(_faucetId);

      const consumeTransactionRequest = client.newConsumeTransactionRequest([
        _noteId,
      ]);
      const prover =
        _withRemoteProver && window.remoteProverUrl != null
          ? window.remoteProverInstance
          : undefined;
      const consumeTransactionUpdate =
        await window.helpers.executeAndApplyTransaction(
          targetAccountId,
          consumeTransactionRequest,
          prover
        );
      await window.helpers.waitForTransaction(
        consumeTransactionUpdate.executedTransaction().id().toHex()
      );

      const changedTargetAccount = await client.getAccount(targetAccountId);

      return {
        transactionId: consumeTransactionUpdate
          .executedTransaction()
          .id()
          .toHex(),
        nonce: consumeTransactionUpdate
          .executedTransaction()
          .accountDelta()
          .nonceDelta()
          .toString(),
        numConsumedNotes: consumeTransactionUpdate
          .executedTransaction()
          .inputNotes()
          .numNotes(),
        targetAccountBalance: changedTargetAccount!
          .vault()
          .getBalance(faucetId)
          .toString(),
      };
    },
    {
      _targetAccountId: targetAccountId,
      _faucetId: faucetId,
      _noteId: noteId,
      _withRemoteProver: withRemoteProver,
    }
  );
};

interface MintAndConsumeTransactionUpdate {
  mintResult: MintTransactionUpdate;
  consumeResult: ConsumeTransactionUpdate;
}

export const mintAndConsumeTransaction = async (
  testingPage: Page,
  targetAccountId: string,
  faucetAccountId: string,
  withRemoteProver: boolean = false,
  sync: boolean = true
): Promise<MintAndConsumeTransactionUpdate> => {
  return await testingPage.evaluate(
    async ({
      _targetAccountId,
      _faucetAccountId,
      _withRemoteProver,
      _sync,
    }) => {
      const client = window.client;

      await client.syncState();

      const targetAccountId = window.AccountId.fromHex(_targetAccountId);
      const faucetAccountId = window.AccountId.fromHex(_faucetAccountId);

      let mintTransactionRequest = await client.newMintTransactionRequest(
        targetAccountId,
        faucetAccountId,
        window.NoteType.Private,
        BigInt(1000)
      );

      const prover =
        _withRemoteProver && window.remoteProverUrl != null
          ? window.remoteProverInstance
          : undefined;

      const mintTransactionUpdate =
        await window.helpers.executeAndApplyTransaction(
          faucetAccountId,
          mintTransactionRequest,
          prover
        );

      let createdNote = mintTransactionUpdate
        .executedTransaction()
        .outputNotes()
        .notes()[0]
        .intoFull();

      if (!createdNote) {
        throw new Error("Created note is undefined");
      }

      let noteAndArgs = new window.NoteAndArgs(createdNote, null);
      let noteAndArgsArray = new window.NoteAndArgsArray([noteAndArgs]);

      let txRequest = new window.TransactionRequestBuilder()
        .withUnauthenticatedInputNotes(noteAndArgsArray)
        .build();

      let consumeTransactionUpdate =
        await window.helpers.executeAndApplyTransaction(
          targetAccountId,
          txRequest,
          prover
        );

      if (_sync) {
        await window.helpers.waitForTransaction(
          consumeTransactionUpdate.executedTransaction().id().toHex()
        );
      }

      const changedTargetAccount = await client.getAccount(targetAccountId);

      return {
        mintResult: {
          transactionId: mintTransactionUpdate
            .executedTransaction()
            .id()
            .toHex(),
          numOutputNotesCreated: mintTransactionUpdate
            .executedTransaction()
            .outputNotes()
            .numNotes(),
          nonce: mintTransactionUpdate
            .executedTransaction()
            .accountDelta()
            .nonceDelta()
            .toString(),
          createdNoteId: mintTransactionUpdate
            .executedTransaction()
            .outputNotes()
            .notes()[0]
            .id()
            .toString(),
        },
        consumeResult: {
          transactionId: consumeTransactionUpdate
            .executedTransaction()
            .id()
            .toHex(),
          nonce: consumeTransactionUpdate
            .executedTransaction()
            .accountDelta()
            .nonceDelta()
            .toString(),
          numConsumedNotes: consumeTransactionUpdate
            .executedTransaction()
            .inputNotes()
            .numNotes(),
          targetAccountBalance: changedTargetAccount!
            .vault()
            .getBalance(faucetAccountId)
            .toString(),
        },
      };
    },
    {
      _targetAccountId: targetAccountId,
      _faucetAccountId: faucetAccountId,
      _withRemoteProver: withRemoteProver,
      _sync: sync,
    }
  );
};

interface SetupWalletFaucetResult {
  accountId: string;
  faucetId: string;
  accountCommitment: string;
}

export const setupWalletAndFaucet = async (
  testingPage: Page
): Promise<SetupWalletFaucetResult> => {
  return await testingPage.evaluate(async () => {
    const client = window.client;
    const account = await client.newWallet(
      window.AccountStorageMode.private(),
      true
    );
    const faucetAccount = await client.newFaucet(
      window.AccountStorageMode.private(),
      false,
      "DAG",
      8,
      BigInt(10000000)
    );

    return {
      accountId: account.id().toString(),
      accountCommitment: account.commitment().toHex(),
      faucetId: faucetAccount.id().toString(),
    };
  });
};

export const getAccount = async (testingPage: Page, accountId: string) => {
  return await testingPage.evaluate(async (_accountId) => {
    const client = window.client;
    const accountId = window.AccountId.fromHex(_accountId);
    const account = await client.getAccount(accountId);
    return {
      id: account?.id().toString(),
      commitment: account?.commitment().toHex(),
      nonce: account?.nonce().toString(),
      vaultCommitment: account?.vault().root().toHex(),
      storageCommitment: account?.storage().commitment().toHex(),
      codeCommitment: account?.code().commitment().toHex(),
    };
  }, accountId);
};

export const syncState = async (testingPage: Page) => {
  return await testingPage.evaluate(async () => {
    const client = window.client;
    const summary = await client.syncState();
    return {
      blockNum: summary.blockNum(),
    };
  });
};
export const clearStore = async (page: Page) => {
  await page.evaluate(async () => {
    // Open a connection to the list of databases
    const databases = await indexedDB.databases();
    for (const db of databases) {
      // Delete each database by name
      if (db.name) {
        indexedDB.deleteDatabase(db.name);
      }
    }
  });
};

// Misc test utils

export const isValidAddress = (address: string) => {
  expect(address.startsWith("0x")).to.be.true;
};

// Constants

export const badHexId =
  "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

export const setupConsumedNote = async (
  page: Page,
  publicNote: boolean = false
) => {
  const { createdNoteId, accountId, faucetId } = await setupMintedNote(
    page,
    publicNote
  );
  await consumeTransaction(page, accountId, faucetId, createdNoteId);

  return {
    consumedNoteId: createdNoteId,
    accountId: accountId,
    faucetId: faucetId,
  };
};

export const getInputNote = async (noteId: string, testingPage: Page) => {
  return await testingPage.evaluate(async (_noteId) => {
    const client = window.client;
    const note = await client.getInputNote(_noteId);
    return {
      noteId: note ? note.id().toString() : undefined,
    };
  }, noteId);
};

// TODO: Figure out a way to easily pass NoteFilters into the tests
export const getInputNotes = async (testingPage: Page) => {
  return await testingPage.evaluate(async () => {
    const client = window.client;
    const filter = new window.NoteFilter(window.NoteFilterTypes.All);
    const notes = await client.getInputNotes(filter);
    return {
      noteIds: notes.map((note) => note.id().toString()),
    };
  });
};

export const setupMintedNote = async (
  page: Page,
  publicNote: boolean = false
) => {
  const { accountId, faucetId } = await setupWalletAndFaucet(page);
  const { createdNoteId } = await mintTransaction(
    page,
    accountId,
    faucetId,
    undefined,
    undefined,
    publicNote
  );

  return { createdNoteId, accountId, faucetId };
};
