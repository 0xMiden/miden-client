import { vi } from "vitest";
import type {
  SignCallback,
  SignerAccountConfig,
  SignerContextValue,
} from "../../context/SignerContext";

// SIGNER CONTEXT MOCKS
// ================================================================================================

/**
 * Creates a mock AccountStorageMode.
 * Matches the SDK's AccountStorageMode interface.
 */
export const createMockAccountStorageMode = (
  mode: "private" | "public" | "network" = "public"
) => ({
  toString: vi.fn(() => mode),
});

/**
 * Creates a mock SignerAccountConfig with sensible defaults.
 * Used for testing signer provider integration.
 */
export function createMockSignerAccountConfig(
  overrides: Partial<SignerAccountConfig> = {}
): SignerAccountConfig {
  return {
    publicKeyCommitment: new Uint8Array(32).fill(0x42),
    accountType: "RegularAccountImmutableCode",
    storageMode: createMockAccountStorageMode("public") as any,
    ...overrides,
  };
}

/**
 * Creates a mock sign callback function.
 * Returns a mock 67-byte signature (typical ECDSA signature size).
 */
export function createMockSignCallback(): SignCallback {
  return vi.fn().mockResolvedValue(new Uint8Array(67).fill(0xab));
}

/**
 * Creates a complete mock SignerContextValue.
 * Used for testing components that depend on SignerContext.
 */
export function createMockSignerContext(
  overrides: Partial<SignerContextValue> = {}
): SignerContextValue {
  return {
    signCb: createMockSignCallback(),
    accountConfig: createMockSignerAccountConfig(),
    storeName: "test_store",
    name: "TestSigner",
    isConnected: true,
    connect: vi.fn().mockResolvedValue(undefined),
    disconnect: vi.fn().mockResolvedValue(undefined),
    ...overrides,
  };
}

/**
 * Creates a disconnected signer context (before user connects).
 */
export function createDisconnectedSignerContext(
  overrides: Partial<SignerContextValue> = {}
): SignerContextValue {
  return createMockSignerContext({
    isConnected: false,
    ...overrides,
  });
}
