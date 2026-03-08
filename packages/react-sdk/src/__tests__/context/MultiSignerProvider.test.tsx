import { describe, it, expect, vi } from "vitest";
import { render, screen, act, renderHook } from "@testing-library/react";
import React, { type ReactNode } from "react";
import { SignerContext, useSigner } from "../../context/SignerContext";
import type { SignerContextValue } from "../../context/SignerContext";
import {
  MultiSignerProvider,
  SignerSlot,
  useMultiSigner,
} from "../../context/MultiSignerProvider";
import {
  createMockSignerContext,
  createDisconnectedSignerContext,
} from "../mocks/signer-context";

// Helper: wraps a signer provider value + SignerSlot
function MockSignerProvider({
  value,
  children,
}: {
  value: SignerContextValue;
  children?: ReactNode;
}) {
  return (
    <SignerContext.Provider value={value}>
      <SignerSlot />
      {children}
    </SignerContext.Provider>
  );
}

// Helper: reads useMultiSigner() and useSigner() and exposes via test ids
function TestConsumer() {
  const multi = useMultiSigner();
  const signer = useSigner();
  return (
    <div>
      <div data-testid="signer-count">{multi?.signers.length ?? 0}</div>
      <div data-testid="active-name">{multi?.activeSigner?.name ?? "none"}</div>
      <div data-testid="forwarded-name">{signer?.name ?? "none"}</div>
      <div data-testid="forwarded-connected">
        {signer?.isConnected ? "true" : "false"}
      </div>
      <div data-testid="signer-names">
        {multi?.signers.map((s) => s.name).join(",") ?? ""}
      </div>
    </div>
  );
}

describe("MultiSignerProvider", () => {
  describe("SignerSlot registration", () => {
    it("registers signer from nearest SignerContext.Provider", async () => {
      const para = createMockSignerContext({ name: "Para", storeName: "para_1" });

      render(
        <MultiSignerProvider>
          <MockSignerProvider value={para} />
          <TestConsumer />
        </MultiSignerProvider>
      );

      expect(screen.getByTestId("signer-count").textContent).toBe("1");
      expect(screen.getByTestId("signer-names").textContent).toBe("Para");
    });

    it("registers multiple distinct signers", async () => {
      const para = createMockSignerContext({ name: "Para", storeName: "para_1" });
      const turnkey = createMockSignerContext({
        name: "Turnkey",
        storeName: "turnkey_1",
      });

      render(
        <MultiSignerProvider>
          <MockSignerProvider value={para} />
          <MockSignerProvider value={turnkey} />
          <TestConsumer />
        </MultiSignerProvider>
      );

      expect(screen.getByTestId("signer-count").textContent).toBe("2");
    });

    it("unregisters on unmount", async () => {
      const para = createMockSignerContext({ name: "Para", storeName: "para_1" });
      const turnkey = createMockSignerContext({
        name: "Turnkey",
        storeName: "turnkey_1",
      });

      const { rerender } = render(
        <MultiSignerProvider>
          <MockSignerProvider value={para} />
          <MockSignerProvider value={turnkey} />
          <TestConsumer />
        </MultiSignerProvider>
      );

      expect(screen.getByTestId("signer-count").textContent).toBe("2");

      // Remove turnkey
      rerender(
        <MultiSignerProvider>
          <MockSignerProvider value={para} />
          <TestConsumer />
        </MultiSignerProvider>
      );

      expect(screen.getByTestId("signer-count").textContent).toBe("1");
      expect(screen.getByTestId("signer-names").textContent).toBe("Para");
    });

    it("updates registration when signer value changes", async () => {
      const disconnected = createDisconnectedSignerContext({
        name: "Para",
        storeName: "para_1",
      });
      const connected = createMockSignerContext({
        name: "Para",
        storeName: "para_1",
        isConnected: true,
      });

      const { rerender } = render(
        <MultiSignerProvider>
          <MockSignerProvider value={disconnected} />
          <TestConsumer />
        </MultiSignerProvider>
      );

      expect(screen.getByTestId("signer-count").textContent).toBe("1");

      rerender(
        <MultiSignerProvider>
          <MockSignerProvider value={connected} />
          <TestConsumer />
        </MultiSignerProvider>
      );

      // Still 1 signer, but the value was updated
      expect(screen.getByTestId("signer-count").textContent).toBe("1");
    });
  });

  describe("useMultiSigner", () => {
    it("returns null outside MultiSignerProvider", () => {
      const { result } = renderHook(() => useMultiSigner());
      expect(result.current).toBeNull();
    });

    it("returns all registered signers", () => {
      const para = createMockSignerContext({ name: "Para", storeName: "para_1" });
      const turnkey = createMockSignerContext({
        name: "Turnkey",
        storeName: "turnkey_1",
      });

      render(
        <MultiSignerProvider>
          <MockSignerProvider value={para} />
          <MockSignerProvider value={turnkey} />
          <TestConsumer />
        </MultiSignerProvider>
      );

      expect(screen.getByTestId("signer-count").textContent).toBe("2");
    });

    it("activeSigner is null initially", () => {
      const para = createMockSignerContext({ name: "Para", storeName: "para_1" });

      render(
        <MultiSignerProvider>
          <MockSignerProvider value={para} />
          <TestConsumer />
        </MultiSignerProvider>
      );

      expect(screen.getByTestId("active-name").textContent).toBe("none");
    });
  });

  describe("connectSigner", () => {
    it("sets active signer and calls connect()", async () => {
      const connect = vi.fn().mockResolvedValue(undefined);
      const para = createDisconnectedSignerContext({
        name: "Para",
        storeName: "para_1",
        connect,
      });

      let multiRef: ReturnType<typeof useMultiSigner>;
      function Capture() {
        multiRef = useMultiSigner();
        return null;
      }

      render(
        <MultiSignerProvider>
          <MockSignerProvider value={para} />
          <Capture />
          <TestConsumer />
        </MultiSignerProvider>
      );

      await act(async () => {
        await multiRef!.connectSigner("Para");
      });

      expect(connect).toHaveBeenCalled();
      expect(screen.getByTestId("active-name").textContent).toBe("Para");
    });

    it("disconnects previous signer first", async () => {
      const paraDisconnect = vi.fn().mockResolvedValue(undefined);
      const para = createMockSignerContext({
        name: "Para",
        storeName: "para_1",
        isConnected: true,
        disconnect: paraDisconnect,
      });
      const turnkeyConnect = vi.fn().mockResolvedValue(undefined);
      const turnkey = createDisconnectedSignerContext({
        name: "Turnkey",
        storeName: "turnkey_1",
        connect: turnkeyConnect,
      });

      let multiRef: ReturnType<typeof useMultiSigner>;
      function Capture() {
        multiRef = useMultiSigner();
        return null;
      }

      render(
        <MultiSignerProvider>
          <MockSignerProvider value={para} />
          <MockSignerProvider value={turnkey} />
          <Capture />
          <TestConsumer />
        </MultiSignerProvider>
      );

      // First connect to Para so it becomes active
      await act(async () => {
        await multiRef!.connectSigner("Para");
      });

      // Now switch to Turnkey
      await act(async () => {
        await multiRef!.connectSigner("Turnkey");
      });

      expect(paraDisconnect).toHaveBeenCalled();
      expect(turnkeyConnect).toHaveBeenCalled();
      expect(screen.getByTestId("active-name").textContent).toBe("Turnkey");
    });

    it("no-op when already connected to target", async () => {
      const connect = vi.fn().mockResolvedValue(undefined);
      const para = createMockSignerContext({
        name: "Para",
        storeName: "para_1",
        isConnected: true,
        connect,
      });

      let multiRef: ReturnType<typeof useMultiSigner>;
      function Capture() {
        multiRef = useMultiSigner();
        return null;
      }

      render(
        <MultiSignerProvider>
          <MockSignerProvider value={para} />
          <Capture />
          <TestConsumer />
        </MultiSignerProvider>
      );

      // Connect first time
      await act(async () => {
        await multiRef!.connectSigner("Para");
      });

      connect.mockClear();

      // Connect again — should be no-op
      await act(async () => {
        await multiRef!.connectSigner("Para");
      });

      expect(connect).not.toHaveBeenCalled();
    });

    it("reverts activeSignerName on connect() failure", async () => {
      const connect = vi.fn().mockRejectedValue(new Error("Auth failed"));
      const para = createDisconnectedSignerContext({
        name: "Para",
        storeName: "para_1",
        connect,
      });

      let multiRef: ReturnType<typeof useMultiSigner>;
      function Capture() {
        multiRef = useMultiSigner();
        return null;
      }

      render(
        <MultiSignerProvider>
          <MockSignerProvider value={para} />
          <Capture />
          <TestConsumer />
        </MultiSignerProvider>
      );

      await act(async () => {
        await expect(multiRef!.connectSigner("Para")).rejects.toThrow(
          "Auth failed"
        );
      });

      expect(screen.getByTestId("active-name").textContent).toBe("none");
    });

    it("re-throws connect() error to caller", async () => {
      const error = new Error("Connection refused");
      const connect = vi.fn().mockRejectedValue(error);
      const para = createDisconnectedSignerContext({
        name: "Para",
        storeName: "para_1",
        connect,
      });

      let multiRef: ReturnType<typeof useMultiSigner>;
      function Capture() {
        multiRef = useMultiSigner();
        return null;
      }

      render(
        <MultiSignerProvider>
          <MockSignerProvider value={para} />
          <Capture />
        </MultiSignerProvider>
      );

      await act(async () => {
        await expect(multiRef!.connectSigner("Para")).rejects.toThrow(
          "Connection refused"
        );
      });
    });
  });

  describe("disconnectSigner", () => {
    it("calls disconnect on active signer and clears activeSigner", async () => {
      const disconnect = vi.fn().mockResolvedValue(undefined);
      const para = createMockSignerContext({
        name: "Para",
        storeName: "para_1",
        isConnected: true,
        disconnect,
      });

      let multiRef: ReturnType<typeof useMultiSigner>;
      function Capture() {
        multiRef = useMultiSigner();
        return null;
      }

      render(
        <MultiSignerProvider>
          <MockSignerProvider value={para} />
          <Capture />
          <TestConsumer />
        </MultiSignerProvider>
      );

      await act(async () => {
        await multiRef!.connectSigner("Para");
      });

      expect(screen.getByTestId("active-name").textContent).toBe("Para");

      await act(async () => {
        await multiRef!.disconnectSigner();
      });

      expect(disconnect).toHaveBeenCalled();
      expect(screen.getByTestId("active-name").textContent).toBe("none");
    });

    it("no-op when no active signer", async () => {
      let multiRef: ReturnType<typeof useMultiSigner>;
      function Capture() {
        multiRef = useMultiSigner();
        return null;
      }

      render(
        <MultiSignerProvider>
          <Capture />
          <TestConsumer />
        </MultiSignerProvider>
      );

      // Should not throw
      await act(async () => {
        await multiRef!.disconnectSigner();
      });

      expect(screen.getByTestId("active-name").textContent).toBe("none");
    });
  });

  describe("forwarded SignerContext", () => {
    it("useSigner() inside MultiSignerProvider sees forwarded active signer", async () => {
      const para = createMockSignerContext({
        name: "Para",
        storeName: "para_1",
        isConnected: true,
      });

      let multiRef: ReturnType<typeof useMultiSigner>;
      function Capture() {
        multiRef = useMultiSigner();
        return null;
      }

      render(
        <MultiSignerProvider>
          <MockSignerProvider value={para} />
          <Capture />
          <TestConsumer />
        </MultiSignerProvider>
      );

      await act(async () => {
        await multiRef!.connectSigner("Para");
      });

      expect(screen.getByTestId("forwarded-name").textContent).toBe("Para");
    });

    it("useSigner() sees null when no signer is active", () => {
      const para = createMockSignerContext({ name: "Para", storeName: "para_1" });

      render(
        <MultiSignerProvider>
          <MockSignerProvider value={para} />
          <TestConsumer />
        </MultiSignerProvider>
      );

      expect(screen.getByTestId("forwarded-name").textContent).toBe("none");
    });

    it("forwarded value has stable reference identity on non-relevant re-renders", async () => {
      const para = createMockSignerContext({
        name: "Para",
        storeName: "para_1",
        isConnected: true,
      });

      const refs: Array<SignerContextValue | null> = [];
      function RefCapture() {
        const signer = useSigner();
        refs.push(signer);
        return null;
      }

      let multiRef: ReturnType<typeof useMultiSigner>;
      function Capture() {
        multiRef = useMultiSigner();
        return null;
      }

      const { rerender } = render(
        <MultiSignerProvider>
          <MockSignerProvider value={para} />
          <Capture />
          <RefCapture />
        </MultiSignerProvider>
      );

      await act(async () => {
        await multiRef!.connectSigner("Para");
      });

      // Re-render with the same signer value (new object, same observable fields).
      // Share accountConfig reference so the register comparison treats it as unchanged.
      const paraClone = createMockSignerContext({
        name: "Para",
        storeName: "para_1",
        isConnected: true,
        accountConfig: para.accountConfig,
      });

      rerender(
        <MultiSignerProvider>
          <MockSignerProvider value={paraClone} />
          <Capture />
          <RefCapture />
        </MultiSignerProvider>
      );

      // The last two non-null refs should be the same object
      const nonNullRefs = refs.filter((r) => r !== null);
      if (nonNullRefs.length >= 2) {
        const last = nonNullRefs[nonNullRefs.length - 1];
        const secondLast = nonNullRefs[nonNullRefs.length - 2];
        expect(last).toBe(secondLast);
      }
    });
  });

  describe("edge cases", () => {
    it("rapid switching discards stale connect via generation counter", async () => {
      let paraResolve: () => void;
      const paraConnect = vi.fn(
        () => new Promise<void>((resolve) => { paraResolve = resolve; })
      );
      const turnkeyConnect = vi.fn().mockResolvedValue(undefined);

      const para = createDisconnectedSignerContext({
        name: "Para",
        storeName: "para_1",
        connect: paraConnect,
      });
      const turnkey = createDisconnectedSignerContext({
        name: "Turnkey",
        storeName: "turnkey_1",
        connect: turnkeyConnect,
      });

      let multiRef: ReturnType<typeof useMultiSigner>;
      function Capture() {
        multiRef = useMultiSigner();
        return null;
      }

      render(
        <MultiSignerProvider>
          <MockSignerProvider value={para} />
          <MockSignerProvider value={turnkey} />
          <Capture />
          <TestConsumer />
        </MultiSignerProvider>
      );

      // Start connecting to Para (will hang on the promise)
      let paraPromise: Promise<void>;
      act(() => {
        paraPromise = multiRef!.connectSigner("Para");
      });

      // Immediately switch to Turnkey before Para resolves
      await act(async () => {
        await multiRef!.connectSigner("Turnkey");
      });

      // Now resolve Para's connect — should be discarded (stale generation)
      await act(async () => {
        paraResolve!();
        await paraPromise!;
      });

      // Turnkey should be active, not Para
      expect(screen.getByTestId("active-name").textContent).toBe("Turnkey");
    });

    it("connectSigner throws for unknown signer name", async () => {
      let multiRef: ReturnType<typeof useMultiSigner>;
      function Capture() {
        multiRef = useMultiSigner();
        return null;
      }

      render(
        <MultiSignerProvider>
          <Capture />
        </MultiSignerProvider>
      );

      await act(async () => {
        await expect(multiRef!.connectSigner("NonExistent")).rejects.toThrow(
          'Signer "NonExistent" not found'
        );
      });
    });

    it("SignerSlot outside MultiSignerProvider does not throw", () => {
      const para = createMockSignerContext({ name: "Para", storeName: "para_1" });

      // Should render without errors — registry is null, so SignerSlot no-ops
      expect(() => {
        render(
          <SignerContext.Provider value={para}>
            <SignerSlot />
          </SignerContext.Provider>
        );
      }).not.toThrow();
    });

    it("disconnectSigner invalidates in-flight connectSigner", async () => {
      let paraResolve: () => void;
      const paraConnect = vi.fn(
        () => new Promise<void>((resolve) => { paraResolve = resolve; })
      );

      const para = createDisconnectedSignerContext({
        name: "Para",
        storeName: "para_1",
        connect: paraConnect,
      });

      let multiRef: ReturnType<typeof useMultiSigner>;
      function Capture() {
        multiRef = useMultiSigner();
        return null;
      }

      render(
        <MultiSignerProvider>
          <MockSignerProvider value={para} />
          <Capture />
          <TestConsumer />
        </MultiSignerProvider>
      );

      // Start connecting (will hang)
      let connectPromise: Promise<void>;
      act(() => {
        connectPromise = multiRef!.connectSigner("Para");
      });

      // Disconnect while connect is in-flight
      await act(async () => {
        await multiRef!.disconnectSigner();
      });

      // Resolve the connect — should be discarded
      await act(async () => {
        paraResolve!();
        await connectPromise!;
      });

      // Should remain disconnected
      expect(screen.getByTestId("active-name").textContent).toBe("none");
    });

    it("works with zero signer providers", () => {
      render(
        <MultiSignerProvider>
          <TestConsumer />
        </MultiSignerProvider>
      );

      expect(screen.getByTestId("signer-count").textContent).toBe("0");
      expect(screen.getByTestId("active-name").textContent).toBe("none");
      expect(screen.getByTestId("forwarded-name").textContent).toBe("none");
    });

    it("two signers get different storeName values", () => {
      const para = createMockSignerContext({
        name: "Para",
        storeName: "para_wallet_abc",
      });
      const turnkey = createMockSignerContext({
        name: "Turnkey",
        storeName: "turnkey_user_xyz",
      });

      let multiRef: ReturnType<typeof useMultiSigner>;
      function Capture() {
        multiRef = useMultiSigner();
        return null;
      }

      render(
        <MultiSignerProvider>
          <MockSignerProvider value={para} />
          <MockSignerProvider value={turnkey} />
          <Capture />
        </MultiSignerProvider>
      );

      const storeNames = multiRef!.signers.map((s) => s.storeName);
      expect(storeNames).toContain("para_wallet_abc");
      expect(storeNames).toContain("turnkey_user_xyz");
      expect(new Set(storeNames).size).toBe(2);
    });
  });
});
