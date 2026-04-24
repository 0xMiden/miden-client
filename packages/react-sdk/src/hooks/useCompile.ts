import { useCallback, useMemo } from "react";
import { CompilerResource, getWasmOrThrow } from "@miden-sdk/miden-sdk/lazy";
import type {
  AccountComponent,
  TransactionScript,
  NoteScript,
  CompileComponentOptions,
  CompileTxScriptOptions,
  CompileNoteScriptOptions,
} from "@miden-sdk/miden-sdk/lazy";
import { useMiden } from "../context/MidenProvider";

export interface UseCompileResult {
  /** Compile MASM source into an AccountComponent. */
  component: (options: CompileComponentOptions) => Promise<AccountComponent>;
  /** Compile MASM source into a TransactionScript. */
  txScript: (options: CompileTxScriptOptions) => Promise<TransactionScript>;
  /** Compile MASM source into a NoteScript. */
  noteScript: (options: CompileNoteScriptOptions) => Promise<NoteScript>;
  /** Whether the underlying client is ready to compile. */
  isReady: boolean;
}

/**
 * Hook for compiling MASM source into `AccountComponent`, `TransactionScript`,
 * or `NoteScript`. Wraps `CompilerResource` from `@miden-sdk/miden-sdk` so the
 * shape is identical to `MidenClient.compile`.
 *
 * @example
 * ```tsx
 * const { noteScript, isReady } = useCompile();
 *
 * const script = await noteScript({
 *   code: noteSource,
 *   libraries: [{ namespace: "my_lib", code: libSource, linking: Linking.Dynamic }],
 * });
 * ```
 */
export function useCompile(): UseCompileResult {
  const { client, isReady } = useMiden();

  const resource = useMemo(
    () =>
      client && isReady ? new CompilerResource(client, getWasmOrThrow) : null,
    [client, isReady]
  );

  const requireResource = useCallback(() => {
    if (!resource) {
      throw new Error("Miden client is not ready");
    }
    return resource;
  }, [resource]);

  const component = useCallback(
    async (options: CompileComponentOptions) =>
      requireResource().component(options),
    [requireResource]
  );

  const txScript = useCallback(
    async (options: CompileTxScriptOptions) =>
      requireResource().txScript(options),
    [requireResource]
  );

  const noteScript = useCallback(
    async (options: CompileNoteScriptOptions) =>
      requireResource().noteScript(options),
    [requireResource]
  );

  return { component, txScript, noteScript, isReady };
}
