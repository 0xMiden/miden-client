export class CompilerResource {
  #inner;
  #getWasm;
  #client;

  constructor(inner, getWasm, client) {
    this.#inner = inner;
    this.#getWasm = getWasm;
    this.#client = client;
  }

  /**
   * Compiles MASM code + slots into an AccountComponent ready for accounts.create().
   *
   * @param {{ code: string, slots: StorageSlot[] }} opts
   * @returns {Promise<AccountComponent>}
   */
  async component({ code, slots }) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();
    const builder = this.#inner.createCodeBuilder();
    const compiled = builder.compileAccountComponentCode(code);
    return wasm.AccountComponent.compile(compiled, slots).withSupportsAllTypes();
  }

  /**
   * Compiles a transaction script, optionally linking named libraries inline.
   *
   * @param {{ code: string, libraries?: Array<{ namespace: string, code: string, linking?: "dynamic" | "static" }> }} opts
   * @returns {Promise<TransactionScript>}
   */
  async txScript({ code, libraries = [] }) {
    this.#client.assertNotTerminated();
    const builder = this.#inner.createCodeBuilder();
    for (const lib of libraries) {
      if (lib && typeof lib.namespace === "string") {
        // Inline { namespace, code, linking? } — build and link automatically
        const built = builder.buildLibrary(lib.namespace, lib.code);
        if (lib.linking === "static") {
          builder.linkStaticLibrary(built);
        } else {
          // Default: "dynamic" — matches existing tutorial behavior
          builder.linkDynamicLibrary(built);
        }
      } else {
        // Pre-built library object — link dynamically
        builder.linkDynamicLibrary(lib);
      }
    }
    return builder.compileTxScript(code);
  }
}
