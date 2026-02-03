import { describe, it, expect } from "vitest";
import { renderHook } from "@testing-library/react";
import React from "react";
import { useSigner, SignerContext } from "../../context/SignerContext";
import { createMockSignerContext } from "../mocks/signer-context";

describe("useSigner", () => {
  describe("when no SignerContext provider is present", () => {
    it("should return null", () => {
      const { result } = renderHook(() => useSigner());

      expect(result.current).toBeNull();
    });
  });

  describe("when SignerContext provider is present", () => {
    it("should return the context value", () => {
      const mockSignerContext = createMockSignerContext();

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <SignerContext.Provider value={mockSignerContext}>
          {children}
        </SignerContext.Provider>
      );

      const { result } = renderHook(() => useSigner(), { wrapper });

      expect(result.current).toBe(mockSignerContext);
    });

    it("should include all required fields", () => {
      const mockSignerContext = createMockSignerContext({
        name: "TestProvider",
        storeName: "test_db",
        isConnected: true,
      });

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <SignerContext.Provider value={mockSignerContext}>
          {children}
        </SignerContext.Provider>
      );

      const { result } = renderHook(() => useSigner(), { wrapper });

      expect(result.current).not.toBeNull();
      expect(result.current!.signCb).toBeDefined();
      expect(result.current!.accountConfig).toBeDefined();
      expect(result.current!.storeName).toBe("test_db");
      expect(result.current!.name).toBe("TestProvider");
      expect(result.current!.isConnected).toBe(true);
      expect(result.current!.connect).toBeDefined();
      expect(result.current!.disconnect).toBeDefined();
    });

    it("should return disconnected state when signer is not connected", () => {
      const mockSignerContext = createMockSignerContext({
        isConnected: false,
      });

      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <SignerContext.Provider value={mockSignerContext}>
          {children}
        </SignerContext.Provider>
      );

      const { result } = renderHook(() => useSigner(), { wrapper });

      expect(result.current).not.toBeNull();
      expect(result.current!.isConnected).toBe(false);
    });

    it("should update when context value changes", () => {
      const initialContext = createMockSignerContext({ isConnected: false });
      const updatedContext = createMockSignerContext({ isConnected: true });

      let contextValue = initialContext;
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <SignerContext.Provider value={contextValue}>
          {children}
        </SignerContext.Provider>
      );

      const { result, rerender } = renderHook(() => useSigner(), { wrapper });

      expect(result.current!.isConnected).toBe(false);

      // Update context
      contextValue = updatedContext;
      rerender();

      expect(result.current!.isConnected).toBe(true);
    });
  });
});
