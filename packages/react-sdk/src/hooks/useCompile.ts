import { useCallback } from "react";
import { AccountComponent, Linking } from "@miden-sdk/miden-sdk";
import type {
  CodeBuilder,
  TransactionScript,
  NoteScript,
  CompileComponentOptions,
  CompileTxScriptOptions,
  CompileNoteScriptOptions,
  CompileTxScriptLibrary,
} from "@miden-sdk/miden-sdk";
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
 * or `NoteScript`. Mirrors `MidenClient.compile` from `@miden-sdk/miden-sdk`.
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

  const requireClient = useCallback(() => {
    if (!client || !isReady) {
      throw new Error("Miden client is not ready");
    }
    return client;
  }, [client, isReady]);

  const linkLibraries = useCallback(
    (builder: CodeBuilder, libraries: CompileTxScriptLibrary[]) => {
      for (const lib of libraries) {
        const built = builder.buildLibrary(lib.namespace, lib.code);
        if (lib.linking === Linking.Static) {
          builder.linkStaticLibrary(built);
        } else {
          builder.linkDynamicLibrary(built);
        }
      }
    },
    []
  );

  const component = useCallback(
    async ({
      code,
      slots = [],
      supportAllTypes = true,
    }: CompileComponentOptions): Promise<AccountComponent> => {
      const c = requireClient();
      const builder = c.createCodeBuilder();
      const compiled = builder.compileAccountComponentCode(code);
      const comp = AccountComponent.compile(compiled, slots);
      return supportAllTypes ? comp.withSupportsAllTypes() : comp;
    },
    [requireClient]
  );

  const txScript = useCallback(
    async ({
      code,
      libraries = [],
    }: CompileTxScriptOptions): Promise<TransactionScript> => {
      const c = requireClient();
      const builder = c.createCodeBuilder();
      linkLibraries(builder, libraries);
      return builder.compileTxScript(code);
    },
    [requireClient, linkLibraries]
  );

  const noteScript = useCallback(
    async ({
      code,
      libraries = [],
    }: CompileNoteScriptOptions): Promise<NoteScript> => {
      const c = requireClient();
      const builder = c.createCodeBuilder();
      linkLibraries(builder, libraries);
      return builder.compileNoteScript(code);
    },
    [requireClient, linkLibraries]
  );

  return { component, txScript, noteScript, isReady };
}
