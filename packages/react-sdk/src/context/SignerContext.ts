import { createContext, useContext } from "react";
import type { AccountStorageMode, AccountComponent } from "@miden-sdk/miden-sdk";

// SIGNER CONTEXT
// ================================================================================================

/**
 * Sign callback for WebClient.createClientWithExternalKeystore.
 * Called when a transaction needs to be signed.
 *
 * @param pubKey - Public key commitment bytes
 * @param signingInputs - Serialized signing inputs
 * @returns Promise resolving to the signature bytes
 */
export type SignCallback = (
  pubKey: Uint8Array,
  signingInputs: Uint8Array
) => Promise<Uint8Array>;

/**
 * Account type for signer accounts.
 * Matches the AccountType enum from the SDK.
 */
export type SignerAccountType =
  | "RegularAccountImmutableCode"
  | "RegularAccountUpdatableCode"
  | "FungibleFaucet"
  | "NonFungibleFaucet";

/**
 * Account configuration provided by the signer.
 * Used to initialize the account in the client store.
 */
export interface SignerAccountConfig {
  /** Public key commitment (for auth component) */
  publicKeyCommitment: Uint8Array;
  /** Account type */
  accountType: SignerAccountType;
  /** Storage mode (public/private/network) */
  storageMode: AccountStorageMode;
  /** Optional seed for deterministic account ID */
  accountSeed?: Uint8Array;
  /** Optional custom account components to include in the account (e.g. from a compiled .masp package) */
  customComponents?: AccountComponent[];
}

/**
 * Context value provided by signer providers (Para, Turnkey, MidenFi, etc.).
 * Includes everything needed for MidenProvider to create an external keystore client
 * and for apps to show connect/disconnect UI.
 */
export interface SignerContextValue {
  /** Sign callback for external keystore */
  signCb: SignCallback;
  /** Account config for initialization (only valid when connected) */
  accountConfig: SignerAccountConfig;
  /** Store name suffix for IndexedDB isolation (e.g., "para_walletId") */
  storeName: string;
  /** Display name for UI (e.g., "Para", "Turnkey", "MidenFi") */
  name: string;
  /** Whether the signer is connected and ready */
  isConnected: boolean;
  /** Connect to the signer (triggers auth flow) */
  connect: () => Promise<void>;
  /** Disconnect from the signer */
  disconnect: () => Promise<void>;
}

/**
 * React context for signer - null when no signer provider is present.
 * Signer providers (ParaSignerProvider, TurnkeySignerProvider, etc.) populate this context.
 */
export const SignerContext = createContext<SignerContextValue | null>(null);

/**
 * Hook for apps and MidenProvider to interact with the current signer.
 * Returns null if no signer provider is present (local keystore mode).
 *
 * @example
 * ```tsx
 * function ConnectButton() {
 *   const signer = useSigner();
 *   if (!signer) return null; // Local keystore mode
 *
 *   const { isConnected, connect, disconnect, name } = signer;
 *   return isConnected
 *     ? <button onClick={disconnect}>Disconnect {name}</button>
 *     : <button onClick={connect}>Connect with {name}</button>;
 * }
 * ```
 */
export function useSigner(): SignerContextValue | null {
  return useContext(SignerContext);
}
