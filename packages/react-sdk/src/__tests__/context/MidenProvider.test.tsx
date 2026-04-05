import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";
import { WasmWebClient as WebClient } from "@miden-sdk/miden-sdk";
import { MidenProvider, useMiden } from "../../context/MidenProvider";
import { useMidenStore } from "../../store/MidenStore";

beforeEach(() => {
  useMidenStore.getState().reset();
  vi.clearAllMocks();
});

function StatusDisplay() {
  const { isReady, isInitializing, error } = useMiden();
  return (
    <div>
      <span data-testid="ready">{String(isReady)}</span>
      <span data-testid="initializing">{String(isInitializing)}</span>
      <span data-testid="error">{error?.message ?? "none"}</span>
    </div>
  );
}

describe("MidenProvider initialization", () => {
  it("should initialize and become ready", async () => {
    render(
      <MidenProvider config={{ rpcUrl: "https://rpc.testnet.miden.io" }}>
        <StatusDisplay />
      </MidenProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId("ready").textContent).toBe("true");
    });

    expect(WebClient.createClient).toHaveBeenCalled();
  });

  it("should initialize and become ready in StrictMode", async () => {
    render(
      <React.StrictMode>
        <MidenProvider config={{ rpcUrl: "https://rpc.testnet.miden.io" }}>
          <StatusDisplay />
        </MidenProvider>
      </React.StrictMode>
    );

    await waitFor(() => {
      expect(screen.getByTestId("ready").textContent).toBe("true");
    });

    expect(WebClient.createClient).toHaveBeenCalled();
  });
});
