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

    if (
      opts?.type === "ImmutableContract" ||
      opts?.type === "MutableContract"
    ) {
      return await this.#createContract(opts, wasm);
    }

    // Default: wallet (mutable or immutable based on type)
    const mutable = resolveAccountMutability(opts?.type);
    const storageMode = resolveStorageMode(opts?.storage ?? "private", wasm);
    const authScheme = resolveAuthScheme(opts?.auth, wasm);
    const seed = opts?.seed ? await hashSeed(opts.seed) : undefined;
    return await this.#inner.newWallet(storageMode, mutable, authScheme, seed);
  }

  async #createContract(opts, wasm) {
    if (!opts.seed)
      throw new Error("Contract creation requires a 'seed' (Uint8Array)");
    if (!opts.auth)
      throw new Error("Contract creation requires an 'auth' (AuthSecretKey)");

    const mutable = opts.type === "MutableContract";
    const accountTypeEnum = mutable
      ? wasm.AccountType.RegularAccountUpdatableCode
      : wasm.AccountType.RegularAccountImmutableCode;
    const storageMode = resolveStorageMode(opts.storage ?? "public", wasm);
    const authComponent =
      wasm.AccountComponent.createAuthComponentFromSecretKey(opts.auth);

    let builder = new wasm.AccountBuilder(opts.seed)
      .accountType(accountTypeEnum)
      .storageMode(storageMode)
      .withAuthComponent(authComponent);

    for (const component of opts.components ?? []) {
      builder = builder.withComponent(component);
    }

    const built = builder.build();
    const account = built.account;
    const accountId = account.id();

    await this.#inner.addAccountSecretKeyToWebStore(accountId, opts.auth);
    try {
      await this.#inner.newAccount(account, false);
    } catch (err) {
      throw new Error(
        `Contract account creation failed after storing secret key for ${accountId.toString()}. ` +
          `The secret key was stored but the account was not. ` +
          `Original error: ${err?.message ?? err}`
      );
    }
    return await this.#inner.getAccount(accountId);
  }

  async getOrImport(ref) {
    this.#client.assertNotTerminated();
    return (await this.get(ref)) ?? (await this.import(ref));
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

    // Early exit for string, Account, and AccountHeader types before property
    // checks, preventing misrouting if a WASM object ever gains a .file or .seed
    // property. Bare AccountId (no .id() method) falls through to the fallback.
    if (typeof input === "string" || typeof input.id === "function") {
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

    // Fallback: treat as AccountRef (string, AccountId, Account, AccountHeader)
    const id = resolveAccountRef(input, wasm);
    await this.#inner.importAccountById(id);
    return await this.#inner.getAccount(id);
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
