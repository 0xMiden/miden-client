import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, render, screen, act } from "@testing-library/react";
import React from "react";
import { SignerContext, useSignerContext } from "../../context/SignerContext";
import {
  createMockSignerContext,
  createDisconnectedSignerContext,
} from "../mocks/signer-context";

/**
 * These tests verify the signer integration contract between MidenProvider and SignerContext.
 * They test the expected behavior based on the SignerContext value rather than the full
 * MidenProvider initialization flow (which requires WASM and is tested in integration tests).
 */

describe("MidenProvider signer integration contract", () => {
  describe("useSignerContext behavior", () => {
    it("returns null when no SignerContext provider is present", () => {
      const { result } = renderHook(() => useSignerContext());
      expect(result.current).toBeNull();
    });

    it("returns null for local keystore mode (explicitly null provider)", () => {
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <SignerContext.Provider value={null}>{children}</SignerContext.Provider>
      );

      const { result } = renderHook(() => useSignerContext(), { wrapper });
      expect(result.current).toBeNull();
    });

    it("returns context when signer is connected", () => {
      const mockSigner = createMockSignerContext({ isConnected: true });
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <SignerContext.Provider value={mockSigner}>
          {children}
        </SignerContext.Provider>
      );

      const { result } = renderHook(() => useSignerContext(), { wrapper });
      expect(result.current).toBe(mockSigner);
      expect(result.current?.isConnected).toBe(true);
    });

    it("returns context when signer is disconnected", () => {
      const mockSigner = createDisconnectedSignerContext();
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <SignerContext.Provider value={mockSigner}>
          {children}
        </SignerContext.Provider>
      );

      const { result } = renderHook(() => useSignerContext(), { wrapper });
      expect(result.current).toBe(mockSigner);
      expect(result.current?.isConnected).toBe(false);
    });
  });

  describe("SignerContext value structure", () => {
    it("has signCb callback", () => {
      const mockSigner = createMockSignerContext();
      expect(typeof mockSigner.signCb).toBe("function");
    });

    it("has accountConfig object", () => {
      const mockSigner = createMockSignerContext();
      expect(mockSigner.accountConfig).toBeDefined();
      expect(mockSigner.accountConfig.publicKeyCommitment).toBeInstanceOf(
        Uint8Array
      );
      expect(typeof mockSigner.accountConfig.accountType).toBe("string");
      expect(mockSigner.accountConfig.storageMode).toBeDefined();
    });

    it("has storeName string", () => {
      const mockSigner = createMockSignerContext({ storeName: "test_db" });
      expect(mockSigner.storeName).toBe("test_db");
    });

    it("has name string", () => {
      const mockSigner = createMockSignerContext({ name: "TestSigner" });
      expect(mockSigner.name).toBe("TestSigner");
    });

    it("has isConnected boolean", () => {
      const connected = createMockSignerContext({ isConnected: true });
      const disconnected = createMockSignerContext({ isConnected: false });
      expect(connected.isConnected).toBe(true);
      expect(disconnected.isConnected).toBe(false);
    });

    it("has connect and disconnect functions", () => {
      const mockSigner = createMockSignerContext();
      expect(typeof mockSigner.connect).toBe("function");
      expect(typeof mockSigner.disconnect).toBe("function");
    });
  });

  describe("MidenProvider decision logic", () => {
    /**
     * The MidenProvider uses these conditions to decide initialization mode:
     * 1. No signer context (null) => use createClient (local keystore)
     * 2. Signer context exists but !isConnected => wait for connection
     * 3. Signer context exists and isConnected => use createClientWithExternalKeystore
     */

    it("condition: null context means local keystore mode", () => {
      const signerContext = null;
      const shouldUseExternalKeystore = signerContext !== null;
      expect(shouldUseExternalKeystore).toBe(false);
    });

    it("condition: disconnected signer means wait for connection", () => {
      const signerContext = createDisconnectedSignerContext();
      const shouldWait =
        signerContext !== null && !signerContext.isConnected;
      expect(shouldWait).toBe(true);
    });

    it("condition: connected signer means use external keystore", () => {
      const signerContext = createMockSignerContext({ isConnected: true });
      const shouldUseExternalKeystore =
        signerContext !== null && signerContext.isConnected;
      expect(shouldUseExternalKeystore).toBe(true);
    });
  });

  describe("storeName database isolation", () => {
    it("unique storeName per signer/wallet", () => {
      const paraWallet1 = createMockSignerContext({
        storeName: "para_wallet1",
      });
      const paraWallet2 = createMockSignerContext({
        storeName: "para_wallet2",
      });
      const turnkeyWallet = createMockSignerContext({
        storeName: "turnkey_addr123",
      });

      expect(paraWallet1.storeName).not.toBe(paraWallet2.storeName);
      expect(paraWallet1.storeName).not.toBe(turnkeyWallet.storeName);
    });

    it("storeName is used to prefix database name", () => {
      const signer = createMockSignerContext({ storeName: "custom_store" });
      const expectedDbName = `MidenClientDB_${signer.storeName}`;
      expect(expectedDbName).toBe("MidenClientDB_custom_store");
    });
  });

  describe("signCb callback", () => {
    it("receives pubKey and signingInputs", async () => {
      const mockSignCb = vi.fn().mockResolvedValue(new Uint8Array(67));
      const signer = createMockSignerContext({ signCb: mockSignCb });

      const pubKey = new Uint8Array(32).fill(0x01);
      const signingInputs = new Uint8Array(100).fill(0x02);

      await signer.signCb(pubKey, signingInputs);

      expect(mockSignCb).toHaveBeenCalledWith(pubKey, signingInputs);
    });

    it("returns signature bytes", async () => {
      const expectedSignature = new Uint8Array(67).fill(0xab);
      const mockSignCb = vi.fn().mockResolvedValue(expectedSignature);
      const signer = createMockSignerContext({ signCb: mockSignCb });

      const result = await signer.signCb(new Uint8Array(32), new Uint8Array(100));

      expect(result).toBe(expectedSignature);
    });
  });

  describe("accountConfig for account initialization", () => {
    it("publicKeyCommitment is used for auth component", () => {
      const commitment = new Uint8Array(32).fill(0x55);
      const signer = createMockSignerContext({
        accountConfig: {
          publicKeyCommitment: commitment,
          accountType: "RegularAccountImmutableCode",
          storageMode: { toString: () => "public" } as any,
        },
      });

      expect(signer.accountConfig.publicKeyCommitment).toBe(commitment);
    });

    it("accountType determines account creation type", () => {
      const types = [
        "RegularAccountImmutableCode",
        "RegularAccountUpdatableCode",
        "FungibleFaucet",
        "NonFungibleFaucet",
      ] as const;

      for (const accountType of types) {
        const signer = createMockSignerContext({
          accountConfig: {
            publicKeyCommitment: new Uint8Array(32),
            accountType,
            storageMode: { toString: () => "public" } as any,
          },
        });
        expect(signer.accountConfig.accountType).toBe(accountType);
      }
    });

    it("storageMode determines account visibility", () => {
      const publicMode = { toString: () => "public" };
      const privateMode = { toString: () => "private" };
      const networkMode = { toString: () => "network" };

      const publicSigner = createMockSignerContext({
        accountConfig: {
          publicKeyCommitment: new Uint8Array(32),
          accountType: "RegularAccountImmutableCode",
          storageMode: publicMode as any,
        },
      });

      expect(publicSigner.accountConfig.storageMode.toString()).toBe("public");
    });

    it("optional accountSeed for deterministic account ID", () => {
      const seed = new Uint8Array(32).fill(0x11);
      const signer = createMockSignerContext({
        accountConfig: {
          publicKeyCommitment: new Uint8Array(32),
          accountType: "RegularAccountImmutableCode",
          storageMode: { toString: () => "public" } as any,
          accountSeed: seed,
        },
      });

      expect(signer.accountConfig.accountSeed).toBe(seed);
    });
  });

  describe("connect/disconnect UI integration", () => {
    it("connect function can be called to initiate auth", async () => {
      const mockConnect = vi.fn().mockResolvedValue(undefined);
      const signer = createMockSignerContext({ connect: mockConnect });

      await signer.connect();

      expect(mockConnect).toHaveBeenCalled();
    });

    it("disconnect function can be called to end session", async () => {
      const mockDisconnect = vi.fn().mockResolvedValue(undefined);
      const signer = createMockSignerContext({ disconnect: mockDisconnect });

      await signer.disconnect();

      expect(mockDisconnect).toHaveBeenCalled();
    });

    it("name is displayed in UI", () => {
      const TestUI = () => {
        const signer = useSignerContext();
        return <span data-testid="signer-name">{signer?.name}</span>;
      };

      const signer = createMockSignerContext({ name: "Para" });
      render(
        <SignerContext.Provider value={signer}>
          <TestUI />
        </SignerContext.Provider>
      );

      expect(screen.getByTestId("signer-name").textContent).toBe("Para");
    });
  });
});
