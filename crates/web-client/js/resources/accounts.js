import {
  resolveAccountRef,
  resolveStorageMode,
  resolveAuthScheme,
  resolveAccountMutability,
  hashSeed,
} from "../utils.js";

export class AccountsResource {
  #inner;
  #getWasm;
  #client;

  constructor(inner, getWasm, client) {
    this.#inner = inner;
    this.#getWasm = getWasm;
    this.#client = client;
  }

  async create(opts) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();

    if (opts?.type === "FungibleFaucet") {
      const storageMode = resolveStorageMode(opts.storage ?? "public", wasm);
      const authScheme = resolveAuthScheme(opts.auth, wasm);
      return await this.#inner.newFaucet(
        storageMode,
        false,
        opts.symbol,
        opts.decimals,
        BigInt(opts.maxSupply),
        authScheme
      );
    }

    // Default: wallet (mutable or immutable based on type)
    const mutable = resolveAccountMutability(opts?.type);
    const storageMode = resolveStorageMode(opts?.storage ?? "private", wasm);
    const authScheme = resolveAuthScheme(opts?.auth, wasm);
    const seed = opts?.seed ? await hashSeed(opts.seed) : undefined;
    return await this.#inner.newWallet(storageMode, mutable, authScheme, seed);
  }

  async get(ref) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();
    const id = resolveAccountRef(ref, wasm);
    const account = await this.#inner.getAccount(id);
    return account ?? null;
  }

  async list() {
    this.#client.assertNotTerminated();
    return await this.#inner.getAccounts();
  }

  async getDetails(ref) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();
    const id = resolveAccountRef(ref, wasm);
    const account = await this.#inner.getAccount(id);
    if (!account) {
      throw new Error(`Account not found: ${id.toString()}`);
    }
    const keys = await this.#inner.getPublicKeyCommitmentsOfAccount(id);
    return {
      account,
      vault: account.vault(),
      storage: account.storage(),
      code: account.code() ?? null,
      keys,
    };
  }

  async getBalance(accountRef, tokenRef) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();
    const accountId = resolveAccountRef(accountRef, wasm);
    const faucetId = resolveAccountRef(tokenRef, wasm);
    const reader = this.#inner.accountReader(accountId);
    return await reader.getBalance(faucetId);
  }

  async import(input) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();

    if (typeof input === "string") {
      // Import by ID (hex or bech32 string)
      const id = resolveAccountRef(input, wasm);
      await this.#inner.importAccountById(id);
      return await this.#inner.getAccount(id);
    }

    if (input.file) {
      // Extract accountId before importAccountFile — WASM consumes the
      // AccountFile by value, invalidating the JS wrapper after the call.
      const accountId =
        typeof input.file.accountId === "function"
          ? input.file.accountId()
          : null;
      await this.#inner.importAccountFile(input.file);
      if (accountId) {
        return await this.#inner.getAccount(accountId);
      }
      throw new Error(
        "Could not determine account ID from AccountFile. " +
          "Ensure the file contains a valid account."
      );
    }

    if (input.seed) {
      // Import public account from seed
      const authScheme = resolveAuthScheme(input.auth, wasm);
      const mutable = resolveAccountMutability(input.type);
      return await this.#inner.importPublicAccountFromSeed(
        input.seed,
        mutable,
        authScheme
      );
    }

    throw new Error(
      "Invalid import input: expected a string, { file }, or { seed }"
    );
  }

  async export(ref) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();
    const id = resolveAccountRef(ref, wasm);
    return await this.#inner.exportAccountFile(id);
  }

  async addAddress(ref, addr) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();
    const id = resolveAccountRef(ref, wasm);
    const address = wasm.Address.fromBech32(addr);
    await this.#inner.insertAccountAddress(id, address);
  }

  async removeAddress(ref, addr) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();
    const id = resolveAccountRef(ref, wasm);
    const address = wasm.Address.fromBech32(addr);
    await this.#inner.removeAccountAddress(id, address);
  }
}
