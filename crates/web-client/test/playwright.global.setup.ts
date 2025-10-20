//@ts-nocheck
import { test as base } from "@playwright/test";
import { MockWebClient } from "../js";

const TEST_SERVER_PORT = 8080;
const MIDEN_NODE_PORT = 57291;
const REMOTE_TX_PROVER_PORT = 50051;

export const test = base.extend<{ forEachTest: void }>({
  forEachTest: [
    async ({ page }, use) => {
      page.on("console", (msg) => {
        if (msg.type() === "debug") {
          console.log(`PAGE DEBUG: ${msg.text()}`);
        }
      });

      page.on("pageerror", (err) => {
        console.error("PAGE ERROR:", err);
      });

      page.on("error", (err) => {
        console.error("PUPPETEER ERROR:", err);
      });

      await page.goto("http://localhost:8080");
      await page.evaluate(
        async ({ MIDEN_NODE_PORT, remoteProverPort }) => {
          const {
            Account,
            AccountBuilder,
            AccountComponent,
            AccountDelta,
            AccountStorageDelta,
            AccountVaultDelta,
            AccountHeader,
            AccountFile,
            AccountId,
            AccountIdArray,
            AccountInterface,
            AccountStorageMode,
            AccountStorageRequirements,
            AccountType,
            Address,
            AddressInterface,
            AdviceMap,
            AuthSecretKey,
            BasicFungibleFaucetComponent,
            ConsumableNoteRecord,
            Felt,
            FeltArray,
            ForeignAccount,
            FungibleAsset,
            FungibleAssetDelta,
            Library,
            NetworkId,
            Note,
            NoteAndArgs,
            NoteAndArgsArray,
            NoteAssets,
            NoteConsumability,
            NoteExecutionHint,
            NoteExecutionMode,
            NoteFilter,
            NoteFilterTypes,
            NoteIdAndArgs,
            NoteIdAndArgsArray,
            NoteInputs,
            NoteMetadata,
            NoteRecipient,
            NoteScript,
            NoteTag,
            NoteType,
            OutputNote,
            OutputNotesArray,
            PublicKey,
            Rpo256,
            SecretKey,
            ScriptBuilder,
            Endpoint,
            RpcClient,
            NoteId,
            Signature,
            SigningInputs,
            SlotAndKeys,
            SlotAndKeysArray,
            StorageMap,
            StorageSlot,
            TestUtils,
            TokenSymbol,
            TransactionFilter,
            TransactionKernel,
            TransactionProver,
            TransactionRequest,
            TransactionResult,
            TransactionRequestBuilder,
            TransactionScript,
            TransactionScriptInputPair,
            TransactionScriptInputPairArray,
            TransactionSummary,
            Word,
            WebClient,
            MockWebClient,
            MidenArrays,
          } = await import("./index.js");
          let rpcUrl = `http://localhost:${MIDEN_NODE_PORT}`;
          let proverUrl = remoteProverPort
            ? `http://localhost:${remoteProverPort}`
            : undefined;
          const client = await WebClient.createClient(rpcUrl, undefined);

          window.client = client;
          window.Account = Account;
          window.AccountBuilder = AccountBuilder;
          window.AccountComponent = AccountComponent;
          window.AccountDelta = AccountDelta;
          window.AccountStorageDelta = AccountStorageDelta;
          window.AccountVaultDelta = AccountVaultDelta;
          window.AccountHeader = AccountHeader;
          window.AccountFile = AccountFile;
          window.AccountId = AccountId;
          window.AccountIdArray = AccountIdArray;
          window.AccountInterface = AccountInterface;
          window.AccountStorageMode = AccountStorageMode;
          window.AccountStorageRequirements = AccountStorageRequirements;
          window.AccountType = AccountType;
          window.Address = Address;
          window.AddressInterface = AddressInterface;
          window.AdviceMap = AdviceMap;
          window.AuthSecretKey = AuthSecretKey;
          window.BasicFungibleFaucetComponent = BasicFungibleFaucetComponent;
          window.ConsumableNoteRecord = ConsumableNoteRecord;
          window.Endpoint = Endpoint;
          window.Felt = Felt;
          window.FeltArray = FeltArray;
          window.ForeignAccount = ForeignAccount;
          window.FungibleAsset = FungibleAsset;
          window.FungibleAssetDelta = FungibleAssetDelta;
          window.Library = Library;
          window.NetworkId = NetworkId;
          window.Note = Note;
          window.NoteAndArgs = NoteAndArgs;
          window.NoteAndArgsArray = NoteAndArgsArray;
          window.NoteAssets = NoteAssets;
          window.NoteConsumability = NoteConsumability;
          window.NoteExecutionHint = NoteExecutionHint;
          window.NoteExecutionMode = NoteExecutionMode;
          window.NoteFilter = NoteFilter;
          window.NoteFilterTypes = NoteFilterTypes;
          window.NoteIdAndArgs = NoteIdAndArgs;
          window.NoteIdAndArgsArray = NoteIdAndArgsArray;
          window.NoteInputs = NoteInputs;
          window.NoteMetadata = NoteMetadata;
          window.NoteRecipient = NoteRecipient;
          window.NoteScript = NoteScript;
          window.NoteId = NoteId;
          window.NoteTag = NoteTag;
          window.NoteType = NoteType;
          window.OutputNote = OutputNote;
          window.OutputNotesArray = OutputNotesArray;
          window.PublicKey = PublicKey;
          window.Rpo256 = Rpo256;
          window.RpcClient = RpcClient;
          window.SecretKey = SecretKey;
          window.Signature = Signature;
          window.SigningInputs = SigningInputs;
          window.SlotAndKeys = SlotAndKeys;
          window.SlotAndKeysArray = SlotAndKeysArray;
          window.StorageMap = StorageMap;
          window.StorageSlot = StorageSlot;
          window.TestUtils = TestUtils;
          window.TokenSymbol = TokenSymbol;
          window.TransactionFilter = TransactionFilter;
          window.TransactionKernel = TransactionKernel;
          window.TransactionProver = TransactionProver;
          window.TransactionRequest = TransactionRequest;
          window.TransactionResult = TransactionResult;
          window.TransactionRequestBuilder = TransactionRequestBuilder;
          window.TransactionScript = TransactionScript;
          window.TransactionScriptInputPair = TransactionScriptInputPair;
          window.TransactionScriptInputPairArray =
            TransactionScriptInputPairArray;
          window.TransactionSummary = TransactionSummary;
          window.WebClient = WebClient;
          window.Word = Word;
          window.MockWebClient = MockWebClient;
          window.MidenArrays = MidenArrays;
          window.ScriptBuilder = ScriptBuilder;

          // Create a namespace for helper functions
          window.helpers = window.helpers || {};

          // Add the remote prover url to window
          window.remoteProverUrl = proverUrl;
          window.rpcUrl = rpcUrl;
          if (window.remoteProverUrl) {
            window.remoteProverInstance =
              window.TransactionProver.newRemoteProver(window.remoteProverUrl);
          }

          window.helpers.waitForTransaction = async (
            transactionId,
            maxWaitTime = 10000,
            delayInterval = 1000
          ) => {
            const client = window.client;
            let timeWaited = 0;
            while (true) {
              if (timeWaited >= maxWaitTime) {
                throw new Error("Timeout waiting for transaction");
              }
              await client.syncState();
              const uncommittedTransactions = await client.getTransactions(
                window.TransactionFilter.uncommitted()
              );
              let uncommittedTransactionIds = uncommittedTransactions.map(
                (transaction) => transaction.id().toHex()
              );
              if (!uncommittedTransactionIds.includes(transactionId)) {
                break;
              }
              await new Promise((r) => setTimeout(r, delayInterval));
              timeWaited += delayInterval;
            }
          };

          window.helpers.waitForBlocks = async (amountOfBlocks) => {
            const client = window.client;
            let currentBlock = await client.getSyncHeight();
            let finalBlock = currentBlock + amountOfBlocks;
            console.log(
              `Current block: ${currentBlock}, waiting for ${amountOfBlocks} blocks...`
            );
            while (true) {
              let syncSummary = await client.syncState();
              console.log(
                `Synced to block ${syncSummary.blockNum()} (syncing until ${finalBlock})`
              );
              if (syncSummary.blockNum() >= finalBlock) {
                return;
              }
              await new Promise((r) => setTimeout(r, 1000));
            }
          };

          window.helpers.refreshClient = async (initSeed) => {
            const client = await WebClient.createClient(rpcUrl, initSeed);
            window.client = client;
            await window.client.syncState();
          };

          window.helpers.parseNetworkId = (networkId) => {
            const map = {
              mm: window.NetworkId.Mainnet,
              mtst: window.NetworkId.Testnet,
              mdev: window.NetworkId.Devnet,
            };
            const parsedNetworkId = map[networkId];
            if (parsedNetworkId === undefined) {
              throw new Error(
                `Invalid network ID: ${networkId}. Expected one of: ${Object.keys(map).join(", ")}`
              );
            }
            return parsedNetworkId;
          };
        },
        {
          MIDEN_NODE_PORT,
          remoteProverPort: process.env.REMOTE_PROVER
            ? REMOTE_TX_PROVER_PORT
            : null,
        }
      );
      await use();
    },
    { auto: true },
  ],
});

export default test;
