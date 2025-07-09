import { expect } from "chai";
import { TransactionProver, WebClient } from "../dist";
import { testingPage } from "./mocha.global.setup.mjs";

interface MintTransactionResult {
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
  targetAccountId: string,
  faucetAccountId: string,
  withRemoteProver: boolean = false,
  sync: boolean = true
): Promise<MintTransactionResult> => {
  return await testingPage.evaluate(
    async (_targetAccountId, _faucetAccountId, _withRemoteProver, _sync) => {
      const client = window.client;

      await client.syncState();

      const targetAccountId = window.AccountId.fromHex(_targetAccountId);
      const faucetAccountId = window.AccountId.fromHex(_faucetAccountId);

      const mintTransactionRequest = client.newMintTransactionRequest(
        targetAccountId,
        faucetAccountId,
        window.NoteType.Private,
        BigInt(1000)
      );
      const mintTransactionResult = await client.newTransaction(
        faucetAccountId,
        mintTransactionRequest
      );
      if (_withRemoteProver && window.remoteProverUrl != null) {
        await client.submitTransaction(
          mintTransactionResult,
          window.remoteProverInstance
        );
      } else {
        await client.submitTransaction(mintTransactionResult);
      }

      if (_sync) {
        await window.helpers.waitForTransaction(
          mintTransactionResult.executedTransaction().id().toHex()
        );
      }

      return {
        transactionId: mintTransactionResult.executedTransaction().id().toHex(),
        numOutputNotesCreated: mintTransactionResult.createdNotes().numNotes(),
        nonce: mintTransactionResult.accountDelta().nonceIncrement().toString(),
        createdNoteId: mintTransactionResult
          .createdNotes()
          .notes()[0]
          .id()
          .toString(),
      };
    },
    targetAccountId,
    faucetAccountId,
    withRemoteProver,
    sync
  );
};

export const getSyncHeight = async () => {
  return await testingPage.evaluate(async () => {
    const client = window.client;
    let summary = await client.syncState();
    return summary.blockNum();
  });
};

export const sendTransaction = async (
  senderAccountId: string,
  targetAccountId: string,
  faucetAccountId: string,
  recallHeight?: number,
  withRemoteProver: boolean = false
) => {
  return testingPage.evaluate(
    async (
      _senderAccountId,
      _targetAccountId,
      _faucetAccountId,
      _recallHeight,
      _withRemoteProver
    ) => {
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

      let mintTransactionResult = await client.newTransaction(
        faucetAccountId,
        mintTransactionRequest
      );
      if (_withRemoteProver && window.remoteProverUrl != null) {
        await client.submitTransaction(
          mintTransactionResult,
          window.remoteProverInstance
        );
      } else {
        await client.submitTransaction(mintTransactionResult);
      }

      let createdNote = mintTransactionResult
        .createdNotes()
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

      let consumeTransactionResult = await client.newTransaction(
        senderAccountId,
        txRequest
      );

      if (_withRemoteProver && window.remoteProverUrl != null) {
        await client.submitTransaction(
          consumeTransactionResult,
          window.remoteProverInstance
        );
      } else {
        await client.submitTransaction(consumeTransactionResult);
      }

      let sendTransactionRequest = client.newSendTransactionRequest(
        senderAccountId,
        targetAccountId,
        faucetAccountId,
        window.NoteType.Public,
        BigInt(100),
        _recallHeight
      );
      let sendTransactionResult = await client.newTransaction(
        senderAccountId,
        sendTransactionRequest
      );
      if (_withRemoteProver && window.remoteProverUrl != null) {
        await client.submitTransaction(
          sendTransactionResult,
          window.remoteProverInstance
        );
      } else {
        await client.submitTransaction(sendTransactionResult);
      }
      let sendCreatedNotes = sendTransactionResult.createdNotes().notes();
      let sendCreatedNoteIds = sendCreatedNotes.map((note) =>
        note.id().toString()
      );

      await window.helpers.waitForTransaction(
        sendTransactionResult.executedTransaction().id().toHex()
      );

      return sendCreatedNoteIds;
    },
    senderAccountId,
    targetAccountId,
    faucetAccountId,
    recallHeight,
    withRemoteProver
  );
};

export interface SwapTransactionResult {
  accountAAssets: { assetId: string; amount: string }[] | undefined;
  accountBAssets: { assetId: string; amount: string }[] | undefined;
}

export const swapTransaction = async (
  accountAId: string,
  accountBId: string,
  assetAFaucetId: string,
  assetAAmount: bigint,
  assetBFaucetId: string,
  assetBAmount: bigint,
  withRemoteProver: boolean = false
): Promise<SwapTransactionResult> => {
  return await testingPage.evaluate(
    async (
      _accountAId,
      _accountBId,
      _assetAFaucetId,
      _assetAAmount,
      _assetBFaucetId,
      _assetBAmount,
      _withRemoteProver
    ) => {
      const client = window.client;

      await client.syncState();

      const accountAId = window.AccountId.fromHex(_accountAId);
      const accountBId = window.AccountId.fromHex(_accountBId);
      const assetAFaucetId = window.AccountId.fromHex(_assetAFaucetId);
      const assetBFaucetId = window.AccountId.fromHex(_assetBFaucetId);

      // Swap transaction

      let swapTransactionRequest = client.newSwapTransactionRequest(
        accountAId,
        assetAFaucetId,
        _assetAAmount,
        assetBFaucetId,
        _assetBAmount,
        window.NoteType.Private
      );

      let expectedOutputNotes = swapTransactionRequest.expectedOutputOwnNotes();
      let expectedPaybackNoteDetails = swapTransactionRequest
        .expectedFutureNotes()
        .map((futureNote) => futureNote.noteDetails);

      let swapTransactionResult = await client.newTransaction(
        accountAId,
        swapTransactionRequest
      );

      if (_withRemoteProver && window.remoteProverUrl != null) {
        await client.submitTransaction(
          swapTransactionResult,
          window.remoteProverInstance
        );
      } else {
        await client.submitTransaction(swapTransactionResult);
      }

      await window.helpers.waitForTransaction(
        swapTransactionResult.executedTransaction().id().toHex()
      );

      // Consuming swap note for account B

      let txRequest1 = client.newConsumeTransactionRequest([
        expectedOutputNotes[0].id().toString(),
      ]);

      let consumeTransaction1Result = await client.newTransaction(
        accountBId,
        txRequest1
      );

      if (_withRemoteProver && window.remoteProverUrl != null) {
        await client.submitTransaction(
          consumeTransaction1Result,
          window.remoteProverInstance
        );
      } else {
        await client.submitTransaction(consumeTransaction1Result);
      }

      await window.helpers.waitForTransaction(
        consumeTransaction1Result.executedTransaction().id().toHex()
      );

      // Consuming payback note for account A

      let txRequest2 = client.newConsumeTransactionRequest([
        expectedPaybackNoteDetails[0].id().toString(),
      ]);

      let consumeTransaction2Result = await client.newTransaction(
        accountAId,
        txRequest2
      );

      if (_withRemoteProver && window.remoteProverUrl != null) {
        await client.submitTransaction(
          consumeTransaction2Result,
          window.remoteProverInstance
        );
      } else {
        await client.submitTransaction(consumeTransaction2Result);
      }

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
    accountAId,
    accountBId,
    assetAFaucetId,
    assetAAmount,
    assetBFaucetId,
    assetBAmount,
    withRemoteProver
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
export const createNewWallet = async ({
  storageMode,
  mutable,
  clientSeed,
  isolatedClient,
  walletSeed,
}: {
  storageMode: StorageMode;
  mutable: boolean;
  clientSeed?: Uint8Array;
  isolatedClient?: boolean;
  walletSeed?: Uint8Array;
}): Promise<NewAccountTestResult> => {
  // Serialize initSeed for Puppeteer
  const serializedClientSeed = clientSeed ? Array.from(clientSeed) : null;
  const serializedWalletSeed = walletSeed ? Array.from(walletSeed) : null;

  return await testingPage.evaluate(
    async (
      _storageMode,
      _mutable,
      _serializedClientSeed,
      _isolatedClient,
      _serializedWalletSeed
    ) => {
      if (_isolatedClient) {
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
        window.AccountStorageMode.tryFromStr(_storageMode);

      const newWallet = await client.newWallet(
        accountStorageMode,
        _mutable,
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
    storageMode,
    mutable,
    serializedClientSeed,
    isolatedClient,
    serializedWalletSeed
  );
};

export const createNewFaucet = async (
  storageMode: StorageMode = StorageMode.PUBLIC,
  nonFungible: boolean = false,
  tokenSymbol: string = "DAG",
  decimals: number = 8,
  maxSupply: bigint = BigInt(10000000)
): Promise<NewAccountTestResult> => {
  return await testingPage.evaluate(
    async (_storageMode, _nonFungible, _tokenSymbol, _decimals, _maxSupply) => {
      const client = window.client;
      const accountStorageMode =
        window.AccountStorageMode.tryFromStr(_storageMode);
      const newFaucet = await client.newFaucet(
        accountStorageMode,
        _nonFungible,
        _tokenSymbol,
        _decimals,
        _maxSupply
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
    storageMode,
    nonFungible,
    tokenSymbol,
    decimals,
    maxSupply
  );
};

export const fundAccountFromFaucet = async (
  accountId: string,
  faucetId: string
) => {
  const mintResult = await mintTransaction(accountId, faucetId);
  return await consumeTransaction(
    accountId,
    faucetId,
    mintResult.createdNoteId
  );
};

export const getAccountBalance = async (
  accountId: string,
  faucetId: string
) => {
  return await testingPage.evaluate(
    async (_accountId, _faucetId) => {
      const client = window.client;
      const account = await client.getAccount(
        window.AccountId.fromHex(_accountId)
      );
      let balance = BigInt(0);
      if (account) {
        balance = account
          .vault()
          .getBalance(window.AccountId.fromHex(_faucetId));
      }
      return balance;
    },
    accountId,
    faucetId
  );
};

interface ConsumeTransactionResult {
  transactionId: string;
  nonce: string | undefined;
  numConsumedNotes: number;
  targetAccountBalanace: string;
}

export const consumeTransaction = async (
  targetAccountId: string,
  faucetId: string,
  noteId: string,
  withRemoteProver: boolean = false
): Promise<ConsumeTransactionResult> => {
  return await testingPage.evaluate(
    async (_targetAccountId, _faucetId, _noteId, _withRemoteProver) => {
      const client = window.client;

      await client.syncState();

      const targetAccountId = window.AccountId.fromHex(_targetAccountId);
      const faucetId = window.AccountId.fromHex(_faucetId);

      const consumeTransactionRequest = client.newConsumeTransactionRequest([
        _noteId,
      ]);
      const consumeTransactionResult = await client.newTransaction(
        targetAccountId,
        consumeTransactionRequest
      );
      if (_withRemoteProver && window.remoteProverUrl != null) {
        await client.submitTransaction(
          consumeTransactionResult,
          window.remoteProverInstance
        );
      } else {
        await client.submitTransaction(consumeTransactionResult);
      }
      await window.helpers.waitForTransaction(
        consumeTransactionResult.executedTransaction().id().toHex()
      );

      const changedTargetAccount = await client.getAccount(targetAccountId);

      return {
        transactionId: consumeTransactionResult
          .executedTransaction()
          .id()
          .toHex(),
        nonce: consumeTransactionResult
          .accountDelta()
          .nonceIncrement()
          .toString(),
        numConsumedNotes: consumeTransactionResult.consumedNotes().numNotes(),
        targetAccountBalanace: changedTargetAccount
          .vault()
          .getBalance(faucetId)
          .toString(),
      };
    },
    targetAccountId,
    faucetId,
    noteId,
    withRemoteProver
  );
};

interface MintAndConsumeTransactionResult {
  mintResult: MintTransactionResult;
  consumeResult: ConsumeTransactionResult;
}

export const mintAndConsumeTransaction = async (
  targetAccountId: string,
  faucetAccountId: string,
  withRemoteProver: boolean = false,
  sync: boolean = true
): Promise<MintAndConsumeTransactionResult> => {
  return await testingPage.evaluate(
    async (_targetAccountId, _faucetAccountId, _withRemoteProver, _sync) => {
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

      const mintTransactionResult = await client.newTransaction(
        faucetAccountId,
        mintTransactionRequest
      );

      if (_withRemoteProver && window.remoteProverUrl != null) {
        await client.submitTransaction(
          mintTransactionResult,
          window.remoteProverInstance
        );
      } else {
        await client.submitTransaction(mintTransactionResult);
      }

      let createdNote = mintTransactionResult
        .createdNotes()
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

      let consumeTransactionResult = await client.newTransaction(
        targetAccountId,
        txRequest
      );

      if (_withRemoteProver && window.remoteProverUrl != null) {
        await client.submitTransaction(
          consumeTransactionResult,
          window.remoteProverInstance
        );
      } else {
        await client.submitTransaction(consumeTransactionResult);
      }

      if (_sync) {
        await window.helpers.waitForTransaction(
          consumeTransactionResult.executedTransaction().id().toHex()
        );
      }

      const changedTargetAccount = await client.getAccount(targetAccountId);

      return {
        mintResult: {
          transactionId: mintTransactionResult
            .executedTransaction()
            .id()
            .toHex(),
          numOutputNotesCreated: mintTransactionResult
            .createdNotes()
            .numNotes(),
          nonce: mintTransactionResult
            .accountDelta()
            .nonceIncrement()
            .toString(),
          createdNoteId: mintTransactionResult
            .createdNotes()
            .notes()[0]
            .id()
            .toString(),
        },
        consumeResult: {
          transactionId: consumeTransactionResult
            .executedTransaction()
            .id()
            .toHex(),
          nonce: consumeTransactionResult
            .accountDelta()
            .nonceIncrement()
            .toString(),
          numConsumedNotes: consumeTransactionResult.consumedNotes().numNotes(),
          targetAccountBalanace: changedTargetAccount
            .vault()
            .getBalance(faucetAccountId)
            .toString(),
        },
      };
    },
    targetAccountId,
    faucetAccountId,
    withRemoteProver,
    sync
  );
};

interface SetupWalletFaucetResult {
  accountId: string;
  faucetId: string;
  accountCommitment: string;
}

export const setupWalletAndFaucet =
  async (): Promise<SetupWalletFaucetResult> => {
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

export const getAccount = async (accountId: string) => {
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

export const syncState = async () => {
  return await testingPage.evaluate(async () => {
    const client = window.client;
    const summary = await client.syncState();
    return {
      blockNum: summary.blockNum(),
    };
  });
};
export const clearStore = async () => {
  await testingPage.evaluate(async () => {
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
