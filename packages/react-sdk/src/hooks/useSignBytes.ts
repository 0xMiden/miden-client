import { useCallback, useState } from "react";
import { useSigner } from "../context/SignerContext";
import type { SignBytesKind } from "../context/SignerContext";

export interface UseSignBytesResult {
  /**
   * Ask the connected signer to sign arbitrary bytes — either a Word
   * (`kind: "word"`) or a serialized SigningInputs payload
   * (`kind: "signingInputs"`). Throws if no signer is connected or the
   * signer does not expose `signBytes`.
   */
  signBytes: (data: Uint8Array, kind: SignBytesKind) => Promise<Uint8Array>;
  isSigning: boolean;
  error: Error | null;
  reset: () => void;
}

/**
 * Hook for arbitrary-byte signing through the connected wallet-backed signer.
 *
 * Generic SDK clients cannot sign — only a signer with key custody can. This
 * hook delegates to `signer.signBytes` and surfaces the standard loading/error
 * state pattern. All shipped signer providers (MidenFi, Para, Turnkey)
 * populate `signBytes`, so dApp code is signer-agnostic.
 *
 * @example
 * ```tsx
 * const { signBytes, isSigning } = useSignBytes();
 * const sig = await signBytes(messageBytes, "signingInputs");
 * ```
 */
export function useSignBytes(): UseSignBytesResult {
  const signer = useSigner();
  const [isSigning, setIsSigning] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const signBytes = useCallback(
    async (data: Uint8Array, kind: SignBytesKind): Promise<Uint8Array> => {
      if (!signer?.signBytes) {
        throw new Error(
          "useSignBytes: no connected signer with signBytes capability"
        );
      }
      setIsSigning(true);
      setError(null);
      try {
        return await signer.signBytes(data, kind);
      } catch (err) {
        const e = err instanceof Error ? err : new Error(String(err));
        setError(e);
        throw e;
      } finally {
        setIsSigning(false);
      }
    },
    [signer]
  );

  const reset = useCallback(() => {
    setIsSigning(false);
    setError(null);
  }, []);

  return { signBytes, isSigning, error, reset };
}
