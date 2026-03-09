import {
  createContext,
  useContext,
  useEffect,
  useRef,
  useState,
  useCallback,
  useMemo,
  type ReactNode,
} from "react";
import {
  SignerContext,
  useSigner,
  type SignerContextValue,
} from "./SignerContext";

// MULTI-SIGNER REGISTRY CONTEXT (internal)
// ================================================================================================

interface MultiSignerRegistryValue {
  register: (value: SignerContextValue) => void;
  unregister: (name: string) => void;
}

const MultiSignerRegistryContext =
  createContext<MultiSignerRegistryValue | null>(null);

// MULTI-SIGNER CONSUMER CONTEXT
// ================================================================================================

export interface MultiSignerContextValue {
  /** All registered signer providers */
  signers: SignerContextValue[];
  /** The currently active signer (null if none selected) */
  activeSigner: SignerContextValue | null;
  /** Switch to a signer by name and call its connect() */
  connectSigner: (name: string) => Promise<void>;
  /** Disconnect the active signer and revert to local keystore mode */
  disconnectSigner: () => Promise<void>;
}

const MultiSignerContext = createContext<MultiSignerContextValue | null>(null);

// MULTI-SIGNER PROVIDER
// ================================================================================================

export function MultiSignerProvider({ children }: { children: ReactNode }) {
  const signersRef = useRef<Map<string, SignerContextValue>>(new Map());
  const [signersSnapshot, setSignersSnapshot] = useState<SignerContextValue[]>(
    []
  );
  const [activeSignerName, setActiveSignerName] = useState<string | null>(null);
  const activeSignerNameRef = useRef<string | null>(null);
  const generationRef = useRef(0);

  const updateSnapshot = useCallback(() => {
    setSignersSnapshot(Array.from(signersRef.current.values()));
  }, []);

  const register = useCallback(
    (value: SignerContextValue) => {
      const prev = signersRef.current.get(value.name);
      signersRef.current.set(value.name, value);

      // Only update snapshot when observable fields change (avoid churn from closure recreation)
      if (
        !prev ||
        prev.name !== value.name ||
        prev.isConnected !== value.isConnected ||
        prev.storeName !== value.storeName ||
        prev.accountConfig !== value.accountConfig
      ) {
        updateSnapshot();
      }
    },
    [updateSnapshot]
  );

  const unregister = useCallback(
    (name: string) => {
      signersRef.current.delete(name);
      setActiveSignerName((current) => (current === name ? null : current));
      updateSnapshot();
    },
    [updateSnapshot]
  );

  const registry = useMemo(
    () => ({ register, unregister }),
    [register, unregister]
  );

  // Active signer from snapshot (for deps that need reactive updates)
  const activeSigner = activeSignerName
    ? (signersSnapshot.find((s) => s.name === activeSignerName) ?? null)
    : null;

  // Stable function wrappers that delegate to the ref at call time.
  // These are placed on forwardedValue (guarded by activeSigner != null),
  // but a stale reference could be called after disconnect — explicit null checks are safer.
  const stableSignCb = useCallback(
    async (pubKey: Uint8Array, signingInputs: Uint8Array) => {
      const name = activeSignerName;
      if (!name) throw new Error("No active signer (signer was disconnected)");
      const signer = signersRef.current.get(name);
      if (!signer) throw new Error(`Signer "${name}" not found in registry`);
      return signer.signCb(pubKey, signingInputs);
    },
    [activeSignerName]
  );

  const stableConnect = useCallback(async () => {
    const name = activeSignerName;
    if (!name) throw new Error("No active signer (signer was disconnected)");
    const signer = signersRef.current.get(name);
    if (!signer) throw new Error(`Signer "${name}" not found in registry`);
    return signer.connect();
  }, [activeSignerName]);

  const stableDisconnect = useCallback(async () => {
    const name = activeSignerName;
    if (!name) throw new Error("No active signer (signer was disconnected)");
    const signer = signersRef.current.get(name);
    if (!signer) throw new Error(`Signer "${name}" not found in registry`);
    return signer.disconnect();
  }, [activeSignerName]);

  // Forwarded value for MidenProvider's SignerContext
  // Keyed on activeSignerName + isConnected + storeName to minimize reference changes
  const forwardedValue = useMemo<SignerContextValue | null>(() => {
    if (!activeSigner) return null;
    return {
      name: activeSigner.name,
      isConnected: activeSigner.isConnected,
      storeName: activeSigner.storeName,
      accountConfig: activeSigner.accountConfig,
      signCb: stableSignCb,
      connect: stableConnect,
      disconnect: stableDisconnect,
    };
  }, [
    activeSigner?.name,
    activeSigner?.isConnected,
    activeSigner?.storeName,
    activeSigner?.accountConfig,
    stableSignCb,
    stableConnect,
    stableDisconnect,
  ]);

  // Helper to update both state and ref in sync
  const setActiveName = useCallback((name: string | null) => {
    activeSignerNameRef.current = name;
    setActiveSignerName(name);
  }, []);

  const connectSigner = useCallback(
    async (name: string) => {
      const currentName = activeSignerNameRef.current;
      const currentSigner = currentName
        ? signersRef.current.get(currentName)
        : null;

      // No-op if already connected to target
      if (currentName === name && currentSigner?.isConnected) return;

      // Validate before setting active — don't transiently point at an invalid name
      const newSigner = signersRef.current.get(name);
      if (!newSigner) throw new Error(`Signer "${name}" not found`);

      const generation = ++generationRef.current;

      // Set active name immediately so MidenProvider sees the new signer
      setActiveName(name);

      // Disconnect old signer (fire-and-forget)
      if (currentSigner?.isConnected) {
        currentSigner.disconnect().catch((err) => {
          console.warn("Failed to disconnect previous signer:", err);
        });
      }

      try {
        await newSigner.connect();

        // Stale check — another connect/disconnect call happened while we awaited
        if (generation !== generationRef.current) return;
      } catch (err) {
        // Stale — don't clobber current activeSignerName
        if (generation !== generationRef.current) return;
        setActiveName(null);
        throw err;
      }
    },
    [setActiveName]
  );

  const disconnectSigner = useCallback(async () => {
    ++generationRef.current; // Invalidate any in-flight connectSigner

    const currentName = activeSignerNameRef.current;
    const signer = currentName ? signersRef.current.get(currentName) : null;

    // Clear active first so MidenProvider sees null → local mode
    setActiveName(null);

    if (signer?.isConnected) {
      signer.disconnect().catch((err) => {
        console.warn("Failed to disconnect signer:", err);
      });
    }
  }, [setActiveName]);

  const multiSignerValue = useMemo<MultiSignerContextValue>(
    () => ({
      signers: signersSnapshot,
      activeSigner,
      connectSigner,
      disconnectSigner,
    }),
    [signersSnapshot, activeSigner, connectSigner, disconnectSigner]
  );

  return (
    <SignerContext.Provider value={forwardedValue}>
      <MultiSignerRegistryContext.Provider value={registry}>
        <MultiSignerContext.Provider value={multiSignerValue}>
          {children}
        </MultiSignerContext.Provider>
      </MultiSignerRegistryContext.Provider>
    </SignerContext.Provider>
  );
}

// SIGNER SLOT
// ================================================================================================

/**
 * Render-less component that registers the nearest ancestor's SignerContext value
 * into the MultiSignerProvider registry.
 *
 * Place one `<SignerSlot />` inside each signer provider:
 * ```tsx
 * <ParaSignerProvider>
 *   <SignerSlot />
 * </ParaSignerProvider>
 * ```
 */
export function SignerSlot(): null {
  const signerValue = useSigner();
  const registry = useContext(MultiSignerRegistryContext);
  const nameRef = useRef<string>();

  // Register on every signerValue change.
  // register() does shallow comparison to avoid snapshot churn.
  useEffect(() => {
    if (!signerValue || !registry) return;
    nameRef.current = signerValue.name;
    registry.register(signerValue);
  }, [signerValue, registry]);

  // Unmount-only cleanup
  useEffect(() => {
    return () => {
      if (nameRef.current && registry) {
        registry.unregister(nameRef.current);
      }
    };
  }, [registry]);

  return null;
}

// HOOK
// ================================================================================================

/**
 * Access the multi-signer context for listing signers, connecting, and disconnecting.
 * Returns null when used outside a `MultiSignerProvider`.
 */
export function useMultiSigner(): MultiSignerContextValue | null {
  return useContext(MultiSignerContext);
}
