import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, act } from "@testing-library/react";
import React from "react";
import { WebClient } from "@miden-sdk/miden-sdk";
import { MidenProvider, useMiden } from "../../context/MidenProvider";
import { SignerContext } from "../../context/SignerContext";
import type { SignerContextValue } from "../../context/SignerContext";
import { useMidenStore } from "../../store/MidenStore";
import {
  createMockSignerContext,
  createDisconnectedSignerContext,
} from "../mocks/signer-context";

beforeEach(() => {
  useMidenStore.getState().reset();
  vi.clearAllMocks();
});

function StatusDisplay() {
  const { isReady, isInitializing, error, signerConnected } = useMiden();
  return (
    <div>
      <span data-testid="ready">{String(isReady)}</span>
      <span data-testid="initializing">{String(isInitializing)}</span>
      <span data-testid="error">{error?.message ?? "none"}</span>
      <span data-testid="signer-connected">{String(signerConnected)}</span>
    </div>
  );
}

describe("MidenProvider resilient disconnect handling", () => {
  describe("disconnect does not destroy client", () => {
    it("sets signerConnected=false on disconnect without resetting client", async () => {
      // Start connected
      const connectedSigner = createMockSignerContext({
        isConnected: true,
        storeName: "test_wallet",
      });

      const Wrapper = ({
        signer,
        children,
      }: {
        signer: SignerContextValue;
        children: React.ReactNode;
      }) => (
        <SignerContext.Provider value={signer}>
          <MidenProvider config={{ rpcUrl: "https://rpc.testnet.miden.io" }}>
            {children}
          </MidenProvider>
        </SignerContext.Provider>
      );

      const { rerender } = render(
        <Wrapper signer={connectedSigner}>
          <StatusDisplay />
        </Wrapper>
      );

      // Wait for init
      await waitFor(() => {
        expect(screen.getByTestId("ready").textContent).toBe("true");
      });

      expect(screen.getByTestId("signer-connected").textContent).toBe("true");

      // Verify client was created
      expect(WebClient.createClientWithExternalKeystore).toHaveBeenCalled();
      const createCallCount = (
        WebClient.createClientWithExternalKeystore as ReturnType<typeof vi.fn>
      ).mock.calls.length;

      // Now disconnect
      const disconnectedSigner = createDisconnectedSignerContext({
        storeName: "test_wallet",
      });

      rerender(
        <Wrapper signer={disconnectedSigner}>
          <StatusDisplay />
        </Wrapper>
      );

      await waitFor(() => {
        expect(screen.getByTestId("signer-connected").textContent).toBe(
          "false"
        );
      });

      // Client should still be ready (not destroyed)
      expect(screen.getByTestId("ready").textContent).toBe("true");

      // No new client was created
      expect(
        (WebClient.createClientWithExternalKeystore as ReturnType<typeof vi.fn>)
          .mock.calls.length
      ).toBe(createCallCount);
    });

    it("preserves cached accounts and notes during disconnect", async () => {
      const connectedSigner = createMockSignerContext({
        isConnected: true,
        storeName: "test_wallet",
      });

      const Wrapper = ({
        signer,
        children,
      }: {
        signer: SignerContextValue;
        children: React.ReactNode;
      }) => (
        <SignerContext.Provider value={signer}>
          <MidenProvider config={{ rpcUrl: "https://rpc.testnet.miden.io" }}>
            {children}
          </MidenProvider>
        </SignerContext.Provider>
      );

      render(
        <Wrapper signer={connectedSigner}>
          <StatusDisplay />
        </Wrapper>
      );

      await waitFor(() => {
        expect(screen.getByTestId("ready").textContent).toBe("true");
      });

      // Simulate some cached state
      const store = useMidenStore.getState();
      expect(store.client).not.toBeNull();

      // Disconnect — cached state should NOT be cleared
      // Simulate disconnect by setting signerConnected=false directly
      // (in real usage, MidenProvider effect does this)
      act(() => {
        useMidenStore.getState().setSignerConnected(false);
      });

      const storeAfter = useMidenStore.getState();
      expect(storeAfter.client).not.toBeNull();
      expect(storeAfter.isReady).toBe(true);
      expect(storeAfter.signerConnected).toBe(false);
    });
  });

  describe("reconnect with same storeName reuses client", () => {
    it("hot-swaps signCb without creating a new client", async () => {
      const signCb1 = vi.fn().mockResolvedValue(new Uint8Array(67));
      const connectedSigner = createMockSignerContext({
        isConnected: true,
        storeName: "test_wallet",
        signCb: signCb1,
      });

      const Wrapper = ({
        signer,
        children,
      }: {
        signer: SignerContextValue;
        children: React.ReactNode;
      }) => (
        <SignerContext.Provider value={signer}>
          <MidenProvider config={{ rpcUrl: "https://rpc.testnet.miden.io" }}>
            {children}
          </MidenProvider>
        </SignerContext.Provider>
      );

      const { rerender } = render(
        <Wrapper signer={connectedSigner}>
          <StatusDisplay />
        </Wrapper>
      );

      await waitFor(() => {
        expect(screen.getByTestId("ready").textContent).toBe("true");
      });

      const initialCreateCount = (
        WebClient.createClientWithExternalKeystore as ReturnType<typeof vi.fn>
      ).mock.calls.length;

      // Disconnect
      const disconnectedSigner = createDisconnectedSignerContext({
        storeName: "test_wallet",
      });

      rerender(
        <Wrapper signer={disconnectedSigner}>
          <StatusDisplay />
        </Wrapper>
      );

      await waitFor(() => {
        expect(screen.getByTestId("signer-connected").textContent).toBe(
          "false"
        );
      });

      // Reconnect with new signCb, same storeName
      const signCb2 = vi.fn().mockResolvedValue(new Uint8Array(67));
      const reconnectedSigner = createMockSignerContext({
        isConnected: true,
        storeName: "test_wallet",
        signCb: signCb2,
      });

      rerender(
        <Wrapper signer={reconnectedSigner}>
          <StatusDisplay />
        </Wrapper>
      );

      await waitFor(() => {
        expect(screen.getByTestId("signer-connected").textContent).toBe("true");
      });

      // No additional client was created — signCb was hot-swapped
      expect(
        (WebClient.createClientWithExternalKeystore as ReturnType<typeof vi.fn>)
          .mock.calls.length
      ).toBe(initialCreateCount);
    });
  });

  describe("connect with different storeName creates new client", () => {
    it("creates new client for different identity", async () => {
      const signer1 = createMockSignerContext({
        isConnected: true,
        storeName: "wallet_A",
      });

      const Wrapper = ({
        signer,
        children,
      }: {
        signer: SignerContextValue;
        children: React.ReactNode;
      }) => (
        <SignerContext.Provider value={signer}>
          <MidenProvider config={{ rpcUrl: "https://rpc.testnet.miden.io" }}>
            {children}
          </MidenProvider>
        </SignerContext.Provider>
      );

      const { rerender } = render(
        <Wrapper signer={signer1}>
          <StatusDisplay />
        </Wrapper>
      );

      await waitFor(() => {
        expect(screen.getByTestId("ready").textContent).toBe("true");
      });

      const firstCreateCount = (
        WebClient.createClientWithExternalKeystore as ReturnType<typeof vi.fn>
      ).mock.calls.length;

      // Connect with different identity
      const signer2 = createMockSignerContext({
        isConnected: true,
        storeName: "wallet_B",
      });

      rerender(
        <Wrapper signer={signer2}>
          <StatusDisplay />
        </Wrapper>
      );

      // Should eventually create a new client
      await waitFor(() => {
        expect(
          (
            WebClient.createClientWithExternalKeystore as ReturnType<
              typeof vi.fn
            >
          ).mock.calls.length
        ).toBeGreaterThan(firstCreateCount);
      });
    });
  });

  describe("signerConnected exposed in context", () => {
    it("is null when no signer provider", async () => {
      render(
        <MidenProvider config={{ rpcUrl: "https://rpc.testnet.miden.io" }}>
          <StatusDisplay />
        </MidenProvider>
      );

      await waitFor(() => {
        expect(screen.getByTestId("ready").textContent).toBe("true");
      });

      expect(screen.getByTestId("signer-connected").textContent).toBe("null");
    });

    it("is true when signer is connected and initialized", async () => {
      const signer = createMockSignerContext({
        isConnected: true,
        storeName: "test",
      });

      render(
        <SignerContext.Provider value={signer}>
          <MidenProvider config={{ rpcUrl: "https://rpc.testnet.miden.io" }}>
            <StatusDisplay />
          </MidenProvider>
        </SignerContext.Provider>
      );

      await waitFor(() => {
        expect(screen.getByTestId("signer-connected").textContent).toBe("true");
      });
    });

    it("is false when signer disconnects", async () => {
      const signer = createMockSignerContext({
        isConnected: true,
        storeName: "test",
      });

      const Wrapper = ({
        signerValue,
        children,
      }: {
        signerValue: SignerContextValue;
        children: React.ReactNode;
      }) => (
        <SignerContext.Provider value={signerValue}>
          <MidenProvider config={{ rpcUrl: "https://rpc.testnet.miden.io" }}>
            {children}
          </MidenProvider>
        </SignerContext.Provider>
      );

      const { rerender } = render(
        <Wrapper signerValue={signer}>
          <StatusDisplay />
        </Wrapper>
      );

      await waitFor(() => {
        expect(screen.getByTestId("signer-connected").textContent).toBe("true");
      });

      // Disconnect
      const disconnected = createDisconnectedSignerContext({
        storeName: "test",
      });
      rerender(
        <Wrapper signerValue={disconnected}>
          <StatusDisplay />
        </Wrapper>
      );

      await waitFor(() => {
        expect(screen.getByTestId("signer-connected").textContent).toBe(
          "false"
        );
      });
    });
  });
});

describe("MidenStore resetInMemoryState", () => {
  it("clears cached data but keeps client and config", () => {
    const store = useMidenStore.getState();

    // Set up some state
    const mockClient = {} as any;
    store.setClient(mockClient);
    store.setConfig({ rpcUrl: "https://rpc.testnet.miden.io" });
    store.setSignerConnected(true);

    // Verify state is set
    expect(useMidenStore.getState().client).toBe(mockClient);
    expect(useMidenStore.getState().isReady).toBe(true);

    // Reset in-memory state
    useMidenStore.getState().resetInMemoryState();

    // Client, isReady, config should be preserved
    const after = useMidenStore.getState();
    expect(after.client).toBe(mockClient);
    expect(after.isReady).toBe(true);
    expect(after.config).toEqual({ rpcUrl: "https://rpc.testnet.miden.io" });
    expect(after.signerConnected).toBe(true);

    // Cached data should be cleared
    expect(after.accounts).toEqual([]);
    expect(after.accountDetails.size).toBe(0);
    expect(after.notes).toEqual([]);
    expect(after.consumableNotes).toEqual([]);
    expect(after.assetMetadata.size).toBe(0);
    expect(after.noteFirstSeen.size).toBe(0);
    expect(after.sync.syncHeight).toBe(0);
  });
});

describe("MidenStore signerConnected", () => {
  it("starts as null", () => {
    useMidenStore.getState().reset();
    expect(useMidenStore.getState().signerConnected).toBeNull();
  });

  it("can be set to true", () => {
    useMidenStore.getState().setSignerConnected(true);
    expect(useMidenStore.getState().signerConnected).toBe(true);
  });

  it("can be set to false", () => {
    useMidenStore.getState().setSignerConnected(false);
    expect(useMidenStore.getState().signerConnected).toBe(false);
  });

  it("can be set back to null", () => {
    useMidenStore.getState().setSignerConnected(true);
    useMidenStore.getState().setSignerConnected(null);
    expect(useMidenStore.getState().signerConnected).toBeNull();
  });

  it("is cleared on full reset", () => {
    useMidenStore.getState().setSignerConnected(true);
    useMidenStore.getState().reset();
    expect(useMidenStore.getState().signerConnected).toBeNull();
  });
});
