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
    newMintTransactionRequest: vi.fn().mockReturnValue({}),
    newSendTransactionRequest: vi.fn().mockReturnValue({}),
    newConsumeTransactionRequest: vi.fn().mockReturnValue({}),
    newSwapTransactionRequest: vi.fn().mockReturnValue({}),
    submitNewTransaction: vi
      .fn()
      .mockResolvedValue({ toString: vi.fn(() => "0xtx") }),
    free: vi.fn(),
  };

  const WebClient = Object.assign(
    vi.fn().mockImplementation(() => mockClient),
    {
      createClient: vi.fn().mockResolvedValue(mockClient),
      createClientWithExternalKeystore: vi.fn().mockResolvedValue(mockClient),
    }
  );

  return {
    WebClient,
    AccountId: {
      fromHex: vi.fn((hex: string) => createMockAccountId(hex)),
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
