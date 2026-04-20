import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";
import {
  WasmWebClient as WebClient,
  TransactionProver,
} from "@miden-sdk/miden-sdk/lazy";
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

// Regression: TLA removal in web-client #2010 (v0.14.2) made WASM init truly
// async, so any wasm-bindgen constructor called before WebClient.createClient
// resolves now throws with "wasm.__wbindgen_malloc is undefined". The provider
// used to build the default TransactionProver inside a render-time useMemo,
// which fired before the client init useEffect — so a non-null `prover` config
// crashed on first render. The mock below simulates that ordering contract:
// TransactionProver constructors throw unless WASM has been "initialized"
// (which we flip inside the mocked WebClient.createClient).
describe("MidenProvider WASM readiness ordering", () => {
  function ProverDisplay() {
    const { isReady, prover } = useMiden();
    return (
      <div>
        <span data-testid="ready">{String(isReady)}</span>
        <span data-testid="has-prover">{String(prover != null)}</span>
      </div>
    );
  }

  it("does not construct the prover before WASM is initialized", async () => {
    let wasmReady = false;

    vi.mocked(TransactionProver.newRemoteProver).mockImplementation(
      (url, timeout) => {
        if (!wasmReady) {
          throw new Error("wasm.__wbindgen_malloc is undefined");
        }
        return {
          type: "remote",
          url,
          timeout,
        } as unknown as TransactionProver;
      }
    );
    vi.mocked(TransactionProver.newLocalProver).mockImplementation(() => {
      if (!wasmReady) {
        throw new Error("wasm.__wbindgen_malloc is undefined");
      }
      return { type: "local" } as unknown as TransactionProver;
    });

    // Flip wasmReady to true inside createClient — the real WebClient.createClient
    // is what triggers WASM init, so anything constructed after it resolves
    // should succeed.
    const createClientMock = vi.mocked(WebClient.createClient);
    const defaultResolved = await createClientMock();
    createClientMock.mockImplementation(async () => {
      wasmReady = true;
      return defaultResolved;
    });

    render(
      <MidenProvider
        config={{
          rpcUrl: "https://rpc.testnet.miden.io",
          prover: "testnet",
        }}
      >
        <ProverDisplay />
      </MidenProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId("ready").textContent).toBe("true");
    });

    await waitFor(() => {
      expect(screen.getByTestId("has-prover").textContent).toBe("true");
    });

    expect(TransactionProver.newRemoteProver).toHaveBeenCalled();
  });
});
