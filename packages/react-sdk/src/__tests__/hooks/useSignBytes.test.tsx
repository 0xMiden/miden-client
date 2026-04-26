import { describe, it, expect, vi } from "vitest";
import { act, renderHook } from "@testing-library/react";
import React from "react";
import { useSignBytes } from "../../hooks/useSignBytes";
import { SignerContext } from "../../context/SignerContext";
import { createMockSignerContext } from "../mocks/signer-context";

describe("useSignBytes", () => {
  it("throws when no signer is connected", async () => {
    const { result } = renderHook(() => useSignBytes());
    await expect(
      result.current.signBytes(new Uint8Array([1, 2, 3]), "word")
    ).rejects.toThrow(/no connected signer/);
  });

  it("throws when signer has no signBytes capability", async () => {
    const ctx = createMockSignerContext({});
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <SignerContext.Provider value={ctx}>{children}</SignerContext.Provider>
    );
    const { result } = renderHook(() => useSignBytes(), { wrapper });
    await expect(
      result.current.signBytes(new Uint8Array([1]), "word")
    ).rejects.toThrow(/no connected signer/);
  });

  it("delegates to signer.signBytes when present", async () => {
    const sig = new Uint8Array([0xab, 0xcd]);
    const signBytesMock = vi.fn().mockResolvedValue(sig);
    const ctx = createMockSignerContext({ signBytes: signBytesMock });
    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <SignerContext.Provider value={ctx}>{children}</SignerContext.Provider>
    );
    const { result } = renderHook(() => useSignBytes(), { wrapper });

    let returned: Uint8Array | undefined;
    await act(async () => {
      returned = await result.current.signBytes(
        new Uint8Array([1, 2, 3]),
        "signingInputs"
      );
    });

    expect(returned).toBe(sig);
    expect(signBytesMock).toHaveBeenCalledWith(
      new Uint8Array([1, 2, 3]),
      "signingInputs"
    );
  });
});
