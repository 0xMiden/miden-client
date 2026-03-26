import { describe, it, expect, vi, beforeEach } from "vitest";
import { initializeSignerAccount } from "../../utils/signerAccount";
import { createMockSignerAccountConfig } from "../mocks/signer-context";

// Create mock builders and SDK types
const mockAccountId = {
  toString: vi.fn(() => "0xaccount123"),
  toHex: vi.fn(() => "0xaccount123"),
};

const mockAccount = {
  id: vi.fn(() => mockAccountId),
};

const mockBuildResult = {
  account: mockAccount,
};

const mockBuilder = {
  withAuthComponent: vi.fn().mockReturnThis(),
  accountType: vi.fn().mockReturnThis(),
  storageMode: vi.fn().mockReturnThis(),
  withBasicWalletComponent: vi.fn().mockReturnThis(),
  withComponent: vi.fn().mockReturnThis(),
  build: vi.fn(() => mockBuildResult),
};

const mockCommitmentWord = {
  toHex: vi.fn(() => "0xcommitment"),
};

// Mock the SDK module
vi.mock("@miden-sdk/miden-sdk", async () => {
  const actual = await vi.importActual("@miden-sdk/miden-sdk");
  return {
    ...actual,
    AccountBuilder: vi.fn(() => mockBuilder),
    AccountComponent: {
      createAuthComponentFromCommitment: vi.fn(() => "mockAuthComponent"),
    },
    Word: {
      deserialize: vi.fn(() => mockCommitmentWord),
    },
    AccountType: {
      RegularAccountImmutableCode: "RegularAccountImmutableCode",
      RegularAccountUpdatableCode: "RegularAccountUpdatableCode",
      FungibleFaucet: "FungibleFaucet",
      NonFungibleFaucet: "NonFungibleFaucet",
    },
  };
});

// Import mocked modules for assertions
import { AccountBuilder, AccountComponent, Word } from "@miden-sdk/miden-sdk";

describe("initializeSignerAccount", () => {
  let mockClient: any;

  beforeEach(() => {
    vi.clearAllMocks();

    mockClient = {
      syncState: vi.fn().mockResolvedValue({}),
      importAccountById: vi.fn().mockRejectedValue(new Error("Not found")),
      getAccount: vi.fn().mockRejectedValue(new Error("Not found")),
      newAccount: vi.fn().mockResolvedValue(undefined),
    };
  });

  describe("state synchronization", () => {
    it("should call syncState first", async () => {
      const config = createMockSignerAccountConfig();

      await initializeSignerAccount(mockClient, config);

      expect(mockClient.syncState).toHaveBeenCalled();
      // syncState should be called before any other operations
      expect(mockClient.syncState.mock.invocationCallOrder[0]).toBeLessThan(
        (AccountBuilder as any).mock.invocationCallOrder[0]
      );
    });
  });

  describe("commitment conversion", () => {
    it("should convert Uint8Array commitment to Word", async () => {
      const commitment = new Uint8Array(32).fill(0x42);
      const config = createMockSignerAccountConfig({
        publicKeyCommitment: commitment,
      });

      await initializeSignerAccount(mockClient, config);

      expect(Word.deserialize).toHaveBeenCalledWith(commitment);
    });
  });

  describe("account building", () => {
    it("should create AccountBuilder with seed", async () => {
      const config = createMockSignerAccountConfig({
        accountSeed: new Uint8Array(32).fill(0x11),
      });

      await initializeSignerAccount(mockClient, config);

      expect(AccountBuilder).toHaveBeenCalledWith(config.accountSeed);
    });

    it("should use default seed when not provided", async () => {
      const config = createMockSignerAccountConfig();
      delete (config as any).accountSeed;

      await initializeSignerAccount(mockClient, config);

      expect(AccountBuilder).toHaveBeenCalledWith(expect.any(Uint8Array));
      const callArg = (AccountBuilder as any).mock.calls[0][0];
      expect(callArg.length).toBe(32);
    });

    it("should set auth component from commitment", async () => {
      const config = createMockSignerAccountConfig();

      await initializeSignerAccount(mockClient, config);

      expect(
        AccountComponent.createAuthComponentFromCommitment
      ).toHaveBeenCalledWith(
        mockCommitmentWord,
        1 // ECDSA auth scheme
      );
      expect(mockBuilder.withAuthComponent).toHaveBeenCalledWith(
        "mockAuthComponent"
      );
    });

    it("should set account type from config", async () => {
      const config = createMockSignerAccountConfig({
        accountType: "RegularAccountUpdatableCode",
      });

      await initializeSignerAccount(mockClient, config);

      expect(mockBuilder.accountType).toHaveBeenCalled();
    });

    it("should set storage mode from config", async () => {
      const mockStorageMode = { toString: () => "public" };
      const config = createMockSignerAccountConfig({
        storageMode: mockStorageMode as any,
      });

      await initializeSignerAccount(mockClient, config);

      expect(mockBuilder.storageMode).toHaveBeenCalledWith(mockStorageMode);
    });

    it("should add basic wallet component", async () => {
      const config = createMockSignerAccountConfig();

      await initializeSignerAccount(mockClient, config);

      expect(mockBuilder.withBasicWalletComponent).toHaveBeenCalled();
    });

    it("should build the account", async () => {
      const config = createMockSignerAccountConfig();

      await initializeSignerAccount(mockClient, config);

      expect(mockBuilder.build).toHaveBeenCalled();
    });
  });

  describe("public account import", () => {
    it("should try importAccountById for public accounts", async () => {
      const config = createMockSignerAccountConfig({
        storageMode: { toString: () => "public" } as any,
      });

      await initializeSignerAccount(mockClient, config);

      expect(mockClient.importAccountById).toHaveBeenCalledWith(mockAccountId);
    });

    it("should try importAccountById for network accounts", async () => {
      const config = createMockSignerAccountConfig({
        storageMode: { toString: () => "network" } as any,
      });

      await initializeSignerAccount(mockClient, config);

      expect(mockClient.importAccountById).toHaveBeenCalledWith(mockAccountId);
    });

    it("should not try importAccountById for private accounts", async () => {
      const config = createMockSignerAccountConfig({
        storageMode: { toString: () => "private" } as any,
      });

      await initializeSignerAccount(mockClient, config);

      expect(mockClient.importAccountById).not.toHaveBeenCalled();
    });

    it("should return account ID after successful import", async () => {
      mockClient.importAccountById.mockResolvedValue(undefined);

      const config = createMockSignerAccountConfig({
        storageMode: { toString: () => "public" } as any,
      });

      const result = await initializeSignerAccount(mockClient, config);

      expect(result).toBe("0xaccount123");
      // Should sync after import
      expect(mockClient.syncState).toHaveBeenCalledTimes(2);
    });
  });

  describe("account creation fallback", () => {
    it("should create account locally if import fails", async () => {
      mockClient.importAccountById.mockRejectedValue(
        new Error("Not found on chain")
      );
      mockClient.getAccount.mockRejectedValue(new Error("Not found locally"));

      const config = createMockSignerAccountConfig({
        storageMode: { toString: () => "public" } as any,
      });

      await initializeSignerAccount(mockClient, config);

      expect(mockClient.newAccount).toHaveBeenCalledWith(mockAccount, false);
    });

    it("should not create if account already exists locally", async () => {
      mockClient.importAccountById.mockRejectedValue(
        new Error("Not found on chain")
      );
      mockClient.getAccount.mockResolvedValue(mockAccount);

      const config = createMockSignerAccountConfig({
        storageMode: { toString: () => "public" } as any,
      });

      await initializeSignerAccount(mockClient, config);

      expect(mockClient.newAccount).not.toHaveBeenCalled();
    });
  });

  describe("custom components", () => {
    it("should add custom components via withComponent", async () => {
      const mockComponent1 = { name: "component1", getProcedures: vi.fn() };
      const mockComponent2 = { name: "component2", getProcedures: vi.fn() };
      const config = createMockSignerAccountConfig({
        customComponents: [mockComponent1, mockComponent2] as any,
      });

      await initializeSignerAccount(mockClient, config);

      expect(mockBuilder.withComponent).toHaveBeenCalledTimes(2);
      expect(mockBuilder.withComponent).toHaveBeenCalledWith(mockComponent1);
      expect(mockBuilder.withComponent).toHaveBeenCalledWith(mockComponent2);
    });

    it("should add a single custom component", async () => {
      const mockComponent = { name: "solo-component", getProcedures: vi.fn() };
      const config = createMockSignerAccountConfig({
        customComponents: [mockComponent] as any,
      });

      await initializeSignerAccount(mockClient, config);

      expect(mockBuilder.withComponent).toHaveBeenCalledTimes(1);
      expect(mockBuilder.withComponent).toHaveBeenCalledWith(mockComponent);
    });

    it("should not call withComponent when no custom components", async () => {
      const config = createMockSignerAccountConfig();

      await initializeSignerAccount(mockClient, config);

      expect(mockBuilder.withComponent).not.toHaveBeenCalled();
    });

    it("should not call withComponent when customComponents is empty", async () => {
      const config = createMockSignerAccountConfig({
        customComponents: [] as any,
      });

      await initializeSignerAccount(mockClient, config);

      expect(mockBuilder.withComponent).not.toHaveBeenCalled();
    });

    it("should not call withComponent when customComponents is undefined", async () => {
      const config = createMockSignerAccountConfig({
        customComponents: undefined,
      });

      await initializeSignerAccount(mockClient, config);

      expect(mockBuilder.withComponent).not.toHaveBeenCalled();
    });

    it("should reject invalid custom components", async () => {
      const config = createMockSignerAccountConfig({
        customComponents: [{ notAComponent: true }] as any,
      });

      await expect(initializeSignerAccount(mockClient, config)).rejects.toThrow(
        "Each entry in customComponents must be an AccountComponent instance"
      );
    });

    it("should add custom components after withBasicWalletComponent", async () => {
      const mockComponent = { name: "custom", getProcedures: vi.fn() };
      const config = createMockSignerAccountConfig({
        customComponents: [mockComponent] as any,
      });

      await initializeSignerAccount(mockClient, config);

      const walletOrder =
        mockBuilder.withBasicWalletComponent.mock.invocationCallOrder[0];
      const componentOrder =
        mockBuilder.withComponent.mock.invocationCallOrder[0];
      expect(walletOrder).toBeLessThan(componentOrder);
    });

    it("should call build after adding custom components", async () => {
      const mockComponent = { name: "custom", getProcedures: vi.fn() };
      const config = createMockSignerAccountConfig({
        customComponents: [mockComponent] as any,
      });

      await initializeSignerAccount(mockClient, config);

      const componentOrder =
        mockBuilder.withComponent.mock.invocationCallOrder[0];
      const buildOrder = mockBuilder.build.mock.invocationCallOrder[0];
      expect(componentOrder).toBeLessThan(buildOrder);
    });

    it("should still build and create account with custom components", async () => {
      mockClient.getAccount.mockRejectedValue(new Error("Not found"));
      const mockComponent = { name: "custom", getProcedures: vi.fn() };
      const config = createMockSignerAccountConfig({
        storageMode: { toString: () => "private" } as any,
        customComponents: [mockComponent] as any,
      });

      const result = await initializeSignerAccount(mockClient, config);

      expect(mockBuilder.build).toHaveBeenCalled();
      expect(mockClient.newAccount).toHaveBeenCalledWith(mockAccount, false);
      expect(result).toBe("0xaccount123");
    });
  });

  describe("return value", () => {
    it("should return account ID as string", async () => {
      const config = createMockSignerAccountConfig();

      const result = await initializeSignerAccount(mockClient, config);

      expect(result).toBe("0xaccount123");
      expect(typeof result).toBe("string");
    });
  });

  describe("sync after account setup", () => {
    it("should sync state after account creation", async () => {
      mockClient.getAccount.mockRejectedValue(new Error("Not found"));

      const config = createMockSignerAccountConfig({
        storageMode: { toString: () => "private" } as any,
      });

      await initializeSignerAccount(mockClient, config);

      // Initial sync + sync after newAccount
      expect(mockClient.syncState).toHaveBeenCalledTimes(2);
    });
  });
});
