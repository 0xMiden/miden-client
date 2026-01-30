import { vi, beforeEach, afterEach } from "vitest";
import { cleanup } from "@testing-library/react";

// Mock the entire @miden-sdk/miden-sdk module before any imports
vi.mock("@miden-sdk/miden-sdk", () => {
  const createMockAccountId = (id: string = "0x1234567890abcdef") => ({
    toString: vi.fn(() => id),
    toHex: vi.fn(() => id),
    isFaucet: vi.fn(() => id.startsWith("0x2")),
    isRegularAccount: vi.fn(() => !id.startsWith("0x2")),
    free: vi.fn(),
  });

  const mockClient = {
    getAccounts: vi.fn().mockResolvedValue([]),
    getAccount: vi.fn().mockResolvedValue(null),
    newWallet: vi.fn().mockResolvedValue({}),
    newFaucet: vi.fn().mockResolvedValue({}),
    syncState: vi.fn().mockResolvedValue({ blockNum: vi.fn(() => 100) }),
    getSyncHeight: vi.fn().mockResolvedValue(100),
    getInputNotes: vi.fn().mockResolvedValue([]),
    getConsumableNotes: vi.fn().mockResolvedValue([]),
    getTransactions: vi.fn().mockResolvedValue([
      {
        id: vi.fn(() => ({ toHex: vi.fn(() => "0xtx") })),
        transactionStatus: vi.fn(() => ({
          isPending: vi.fn(() => false),
          isCommitted: vi.fn(() => true),
          isDiscarded: vi.fn(() => false),
        })),
      },
    ]),
    newMintTransactionRequest: vi.fn().mockReturnValue({}),
    newSendTransactionRequest: vi.fn().mockReturnValue({}),
    newConsumeTransactionRequest: vi.fn().mockReturnValue({}),
    newSwapTransactionRequest: vi.fn().mockReturnValue({}),
    submitNewTransaction: vi
      .fn()
      .mockResolvedValue({ toString: vi.fn(() => "0xtx") }),
    executeTransaction: vi.fn().mockResolvedValue({}),
    proveTransaction: vi.fn().mockResolvedValue({}),
    submitProvenTransaction: vi.fn().mockResolvedValue(0),
    applyTransaction: vi.fn().mockResolvedValue({}),
    sendPrivateNote: vi.fn().mockResolvedValue(undefined),
    importAccountFile: vi.fn().mockResolvedValue("Imported account"),
    importAccountById: vi.fn().mockResolvedValue(undefined),
    importPublicAccountFromSeed: vi.fn().mockResolvedValue({}),
    exportAccountFile: vi
      .fn()
      .mockResolvedValue({ serialize: () => new Uint8Array() }),
    free: vi.fn(),
  };

  const WebClient = Object.assign(
    vi.fn().mockImplementation(() => mockClient),
    {
      createClient: vi.fn().mockResolvedValue(mockClient),
      createClientWithExternalKeystore: vi.fn().mockResolvedValue(mockClient),
    }
  );

  class Endpoint {
    constructor(_url?: string) {}
    static testnet() {
      return new Endpoint();
    }
  }

  class RpcClient {
    constructor(_endpoint: unknown) {}
    getAccountDetails = vi.fn().mockResolvedValue({ account: () => null });
  }

  return {
    WebClient,
    AccountId: {
      fromHex: vi.fn((hex: string) => createMockAccountId(hex)),
      fromBech32: vi.fn((bech32: string) => createMockAccountId(bech32)),
    },
    Address: {
      fromBech32: vi.fn((bech32: string) => ({
        accountId: vi.fn(() => createMockAccountId(bech32)),
        toString: vi.fn(() => bech32),
      })),
      fromAccountId: vi.fn(
        (accountId: ReturnType<typeof createMockAccountId>) => ({
          accountId: vi.fn(() => accountId),
          toString: vi.fn(() => accountId.toString()),
        })
      ),
    },
    Endpoint,
    RpcClient,
    BasicFungibleFaucetComponent: {
      fromAccount: vi.fn(() => ({
        symbol: vi.fn(() => ({ toString: () => "TKN" })),
        decimals: vi.fn(() => 0),
      })),
    },
    NoteId: {
      fromHex: vi.fn((hex: string) => ({ toString: () => hex })),
    },
    AccountStorageMode: {
      private: vi.fn(() => ({ type: "private" })),
      public: vi.fn(() => ({ type: "public" })),
      network: vi.fn(() => ({ type: "network" })),
    },
    NoteType: {
      Private: 2,
      Encrypted: 3,
      Public: 1,
    },
    Note: {
      createP2IDNote: vi.fn(
        (
          sender: ReturnType<typeof createMockAccountId>,
          receiver: ReturnType<typeof createMockAccountId>,
          assets: unknown,
          noteType: number,
          attachment: unknown
        ) => ({
          id: vi.fn(() => ({ toString: () => "0xnote" })),
          sender,
          receiver,
          assets,
          noteType,
          attachment,
        })
      ),
    },
    NoteAssets: class NoteAssets {
      assets: unknown[];
      constructor(assets: unknown[]) {
        this.assets = assets;
      }
    },
    FungibleAsset: class FungibleAsset {
      faucetId: ReturnType<typeof createMockAccountId>;
      amount: bigint;
      constructor(
        faucetId: ReturnType<typeof createMockAccountId>,
        amount: bigint
      ) {
        this.faucetId = faucetId;
        this.amount = amount;
      }
    },
    NoteAttachment: class NoteAttachment {},
    OutputNoteArray: class OutputNoteArray {
      notes: unknown[];
      constructor(notes: unknown[]) {
        this.notes = notes;
      }
    },
    OutputNote: {
      full: vi.fn((note: unknown) => ({ note })),
    },
    NoteAndArgs: class NoteAndArgs {
      note: unknown;
      args: unknown;
      constructor(note: unknown, args: unknown) {
        this.note = note;
        this.args = args;
      }
    },
    NoteAndArgsArray: class NoteAndArgsArray {
      notes: unknown[];
      constructor(notes: unknown[]) {
        this.notes = notes;
      }
    },
    TransactionRequestBuilder: class TransactionRequestBuilder {
      withOwnOutputNotes = vi.fn(() => this);
      withInputNotes = vi.fn(() => this);
      build = vi.fn(() => ({}));
    },
    NoteFilter: vi.fn().mockImplementation(() => ({
      free: vi.fn(),
    })),
    NoteFilterTypes: {
      All: 0,
      Consumed: 1,
      Committed: 2,
      Expected: 3,
      Processing: 4,
      List: 5,
      Unique: 6,
      Nullifiers: 7,
      Unverified: 8,
    },
    TransactionFilter: {
      uncommitted: vi.fn(() => ({})),
      ids: vi.fn((ids: unknown) => ({ ids })),
    },
    AccountFile: class AccountFile {
      account() {
        return {};
      }
      accountId() {
        return createMockAccountId("0ximported");
      }
      authSecretKeyCount() {
        return 1;
      }
      serialize() {
        return new Uint8Array();
      }
      static deserialize() {
        return new AccountFile();
      }
    },
  };
});

// Cleanup after each test
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

// Reset modules before each test
beforeEach(() => {
  vi.resetModules();
});

// Mock ResizeObserver for jsdom
(globalThis as typeof globalThis & { ResizeObserver: unknown }).ResizeObserver =
  vi.fn().mockImplementation(() => ({
    observe: vi.fn(),
    unobserve: vi.fn(),
    disconnect: vi.fn(),
  }));
