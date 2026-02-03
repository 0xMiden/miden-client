import { describe, it, expect } from "vitest";
import { renderHook, render, screen } from "@testing-library/react";
import React from "react";
import {
  SignerContext,
  useSignerContext,
  useSigner,
} from "../../context/SignerContext";
import { createMockSignerContext } from "../mocks/signer-context";

describe("SignerContext", () => {
  describe("SignerContext.Provider", () => {
    it("should pass value to children", () => {
      const mockContext = createMockSignerContext({ name: "TestSigner" });

      const TestComponent = () => {
        const signer = useSigner();
        return <div data-testid="signer-name">{signer?.name ?? "none"}</div>;
      };

      render(
        <SignerContext.Provider value={mockContext}>
          <TestComponent />
        </SignerContext.Provider>
      );

      expect(screen.getByTestId("signer-name").textContent).toBe("TestSigner");
    });

    it("should allow null value for local keystore mode", () => {
      const TestComponent = () => {
        const signer = useSigner();
        return (
          <div data-testid="signer-status">
            {signer ? "has-signer" : "no-signer"}
          </div>
        );
      };

      render(
        <SignerContext.Provider value={null}>
          <TestComponent />
        </SignerContext.Provider>
      );

      expect(screen.getByTestId("signer-status").textContent).toBe("no-signer");
    });
  });

  describe("useSignerContext", () => {
    it("should return null when outside provider", () => {
      const { result } = renderHook(() => useSignerContext());

      expect(result.current).toBeNull();
    });

    it("should return context value when inside provider", () => {
      const mockContext = createMockSignerContext({ name: "InternalSigner" });

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <SignerContext.Provider value={mockContext}>
          {children}
        </SignerContext.Provider>
      );

      const { result } = renderHook(() => useSignerContext(), { wrapper });

      expect(result.current).toBe(mockContext);
      expect(result.current?.name).toBe("InternalSigner");
    });

    it("should return null when provider has null value", () => {
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <SignerContext.Provider value={null}>{children}</SignerContext.Provider>
      );

      const { result } = renderHook(() => useSignerContext(), { wrapper });

      expect(result.current).toBeNull();
    });
  });

  describe("nested providers", () => {
    it("should use the innermost provider value", () => {
      const outerContext = createMockSignerContext({ name: "OuterSigner" });
      const innerContext = createMockSignerContext({ name: "InnerSigner" });

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <SignerContext.Provider value={outerContext}>
          <SignerContext.Provider value={innerContext}>
            {children}
          </SignerContext.Provider>
        </SignerContext.Provider>
      );

      const { result } = renderHook(() => useSigner(), { wrapper });

      expect(result.current?.name).toBe("InnerSigner");
    });

    it("should allow inner provider to be null (override with local keystore)", () => {
      const outerContext = createMockSignerContext({ name: "OuterSigner" });

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <SignerContext.Provider value={outerContext}>
          <SignerContext.Provider value={null}>
            {children}
          </SignerContext.Provider>
        </SignerContext.Provider>
      );

      const { result } = renderHook(() => useSigner(), { wrapper });

      expect(result.current).toBeNull();
    });
  });

  describe("context value structure", () => {
    it("should provide signCb callback", async () => {
      const mockContext = createMockSignerContext();

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <SignerContext.Provider value={mockContext}>
          {children}
        </SignerContext.Provider>
      );

      const { result } = renderHook(() => useSigner(), { wrapper });

      const pubKey = new Uint8Array(32);
      const signingInputs = new Uint8Array(100);
      const signature = await result.current!.signCb(pubKey, signingInputs);

      expect(signature).toBeInstanceOf(Uint8Array);
      expect(mockContext.signCb).toHaveBeenCalledWith(pubKey, signingInputs);
    });

    it("should provide accountConfig with required fields", () => {
      const mockContext = createMockSignerContext();

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <SignerContext.Provider value={mockContext}>
          {children}
        </SignerContext.Provider>
      );

      const { result } = renderHook(() => useSigner(), { wrapper });

      const config = result.current!.accountConfig;
      expect(config.publicKeyCommitment).toBeInstanceOf(Uint8Array);
      expect(config.accountType).toBeDefined();
      expect(config.storageMode).toBeDefined();
    });

    it("should provide connect and disconnect functions", async () => {
      const mockContext = createMockSignerContext();

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <SignerContext.Provider value={mockContext}>
          {children}
        </SignerContext.Provider>
      );

      const { result } = renderHook(() => useSigner(), { wrapper });

      await result.current!.connect();
      expect(mockContext.connect).toHaveBeenCalled();

      await result.current!.disconnect();
      expect(mockContext.disconnect).toHaveBeenCalled();
    });
  });
});
