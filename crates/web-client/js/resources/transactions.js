import { resolveAccountRef, resolveNoteType } from "../utils.js";

export class TransactionsResource {
  #inner;
  #getWasm;
  #client;

  constructor(inner, getWasm, client) {
    this.#inner = inner;
    this.#getWasm = getWasm;
    this.#client = client;
  }

  async send(opts) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();

    const accountId = resolveAccountRef(opts.account, wasm);
    const targetId = resolveAccountRef(opts.to, wasm);
    const faucetId = resolveAccountRef(opts.token, wasm);
    const noteType = resolveNoteType(opts.type, wasm);
    const amount = BigInt(opts.amount);

    const request = await this.#inner.newSendTransactionRequest(
      accountId,
      targetId,
      faucetId,
      noteType,
      amount,
      opts.reclaimAfter,
      opts.timelockUntil
    );

    const txId = await this.#submitOrSubmitWithProver(
      accountId,
      request,
      opts.prover
    );

    if (opts.waitForConfirmation) {
      await this.waitFor(txId.toHex(), { timeout: opts.timeout });
    }

    return txId;
  }

  async mint(opts) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();

    const accountId = resolveAccountRef(opts.account, wasm);
    const targetId = resolveAccountRef(opts.to, wasm);
    const noteType = resolveNoteType(opts.type, wasm);
    const amount = BigInt(opts.amount);

    // WASM signature: newMintTransactionRequest(target, faucet, noteType, amount)
    const request = await this.#inner.newMintTransactionRequest(
      targetId,
      accountId,
      noteType,
      amount
    );

    const txId = await this.#submitOrSubmitWithProver(
      accountId,
      request,
      opts.prover
    );

    if (opts.waitForConfirmation) {
      await this.waitFor(txId.toHex(), { timeout: opts.timeout });
    }

    return txId;
  }

  async consume(opts) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();

    const accountId = resolveAccountRef(opts.account, wasm);

    // Normalize to array
    const noteInputs = Array.isArray(opts.notes) ? opts.notes : [opts.notes];
    const notes = await Promise.all(
      noteInputs.map((input) => this.#resolveNoteInput(input))
    );

    const request = await this.#inner.newConsumeTransactionRequest(notes);

    const txId = await this.#submitOrSubmitWithProver(
      accountId,
      request,
      opts.prover
    );

    if (opts.waitForConfirmation) {
      await this.waitFor(txId.toHex(), { timeout: opts.timeout });
    }

    return txId;
  }

  async consumeAll(opts) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();

    const accountId = resolveAccountRef(opts.account, wasm);
    const consumable = await this.#inner.getConsumableNotes(accountId);

    if (!consumable || consumable.length === 0) {
      return { txId: null, consumed: 0, remaining: 0 };
    }

    const total = consumable.length;
    const toConsume = opts.maxNotes
      ? consumable.slice(0, opts.maxNotes)
      : consumable;
    const notes = toConsume.map((c) => c.inputNoteRecord().toNote());

    const request = await this.#inner.newConsumeTransactionRequest(notes);

    const txId = await this.#submitOrSubmitWithProver(
      accountId,
      request,
      opts.prover
    );

    if (opts.waitForConfirmation) {
      await this.waitFor(txId.toHex(), { timeout: opts.timeout });
    }

    return {
      txId,
      consumed: toConsume.length,
      remaining: total - toConsume.length,
    };
  }

  async swap(opts) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();

    const accountId = resolveAccountRef(opts.account, wasm);
    const offeredFaucetId = resolveAccountRef(opts.offer.token, wasm);
    const requestedFaucetId = resolveAccountRef(opts.request.token, wasm);
    const noteType = resolveNoteType(opts.type, wasm);
    const paybackNoteType = resolveNoteType(opts.paybackType, wasm);

    const request = await this.#inner.newSwapTransactionRequest(
      accountId,
      offeredFaucetId,
      BigInt(opts.offer.amount),
      requestedFaucetId,
      BigInt(opts.request.amount),
      noteType,
      paybackNoteType
    );

    const txId = await this.#submitOrSubmitWithProver(
      accountId,
      request,
      opts.prover
    );

    if (opts.waitForConfirmation) {
      await this.waitFor(txId.toHex(), { timeout: opts.timeout });
    }

    return txId;
  }

  async mintAndConsume(opts) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();

    const faucetId = resolveAccountRef(opts.faucet, wasm);
    const targetId = resolveAccountRef(opts.to, wasm);
    const noteType = resolveNoteType(opts.type, wasm);
    const amount = BigInt(opts.amount);

    // Step 1: Mint
    let mintTxId;
    try {
      const mintRequest = await this.#inner.newMintTransactionRequest(
        targetId,
        faucetId,
        noteType,
        amount
      );
      mintTxId = await this.#submitOrSubmitWithProver(
        faucetId,
        mintRequest,
        opts.prover
      );
    } catch (original) {
      const err = new Error(original.message);
      err.step = "mint";
      err.cause = original;
      throw err;
    }

    // Step 2: Wait for mint confirmation
    try {
      await this.waitFor(mintTxId.toHex());
    } catch (original) {
      const err = new Error(original.message);
      err.step = "sync";
      err.cause = original;
      throw err;
    }

    // Step 3: Consume
    try {
      const consumable = await this.#inner.getConsumableNotes(targetId);
      const notes = consumable.map((c) => c.inputNoteRecord().toNote());

      if (notes.length === 0) {
        throw new Error("No consumable notes found after mint");
      }

      const consumeRequest =
        await this.#inner.newConsumeTransactionRequest(notes);
      await this.#submitOrSubmitWithProver(
        targetId,
        consumeRequest,
        opts.prover
      );
    } catch (original) {
      if (original.step) throw original;
      const err = new Error(original.message);
      err.step = "consume";
      err.cause = original;
      throw err;
    }

    return mintTxId;
  }

  async preview(opts) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();

    let accountId;
    let request;

    switch (opts.operation) {
      case "send": {
        accountId = resolveAccountRef(opts.account, wasm);
        const targetId = resolveAccountRef(opts.to, wasm);
        const faucetId = resolveAccountRef(opts.token, wasm);
        const noteType = resolveNoteType(opts.type, wasm);
        request = await this.#inner.newSendTransactionRequest(
          accountId,
          targetId,
          faucetId,
          noteType,
          BigInt(opts.amount),
          opts.reclaimAfter,
          opts.timelockUntil
        );
        break;
      }
      case "mint": {
        accountId = resolveAccountRef(opts.account, wasm);
        const targetId = resolveAccountRef(opts.to, wasm);
        const noteType = resolveNoteType(opts.type, wasm);
        request = await this.#inner.newMintTransactionRequest(
          targetId,
          accountId,
          noteType,
          BigInt(opts.amount)
        );
        break;
      }
      case "consume": {
        accountId = resolveAccountRef(opts.account, wasm);
        const noteInputs = Array.isArray(opts.notes)
          ? opts.notes
          : [opts.notes];
        const notes = await Promise.all(
          noteInputs.map((input) => this.#resolveNoteInput(input))
        );
        request = await this.#inner.newConsumeTransactionRequest(notes);
        break;
      }
      case "swap": {
        accountId = resolveAccountRef(opts.account, wasm);
        const offeredFaucetId = resolveAccountRef(opts.offer.token, wasm);
        const requestedFaucetId = resolveAccountRef(opts.request.token, wasm);
        const noteType = resolveNoteType(opts.type, wasm);
        const paybackNoteType = resolveNoteType(opts.paybackType, wasm);
        request = await this.#inner.newSwapTransactionRequest(
          accountId,
          offeredFaucetId,
          BigInt(opts.offer.amount),
          requestedFaucetId,
          BigInt(opts.request.amount),
          noteType,
          paybackNoteType
        );
        break;
      }
      default:
        throw new Error(`Unknown preview operation: ${opts.operation}`);
    }

    return await this.#inner.executeForSummary(accountId, request);
  }

  async submit(account, request, opts) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();
    const accountId = resolveAccountRef(account, wasm);
    return await this.#submitOrSubmitWithProver(
      accountId,
      request,
      opts?.prover
    );
  }

  async list(query) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();

    let filter;
    if (!query) {
      filter = wasm.TransactionFilter.all();
    } else if (query.status === "uncommitted") {
      filter = wasm.TransactionFilter.uncommitted();
    } else if (query.ids) {
      const txIds = query.ids.map((id) => wasm.TransactionId.fromHex(id));
      filter = wasm.TransactionFilter.ids(txIds);
    } else if (query.expiredBefore !== undefined) {
      filter = wasm.TransactionFilter.expiredBefore(query.expiredBefore);
    } else {
      filter = wasm.TransactionFilter.all();
    }

    return await this.#inner.getTransactions(filter);
  }

  async waitFor(txId, opts) {
    this.#client.assertNotTerminated();
    const timeout = opts?.timeout ?? 60_000;
    const interval = opts?.interval ?? 5_000;
    const start = Date.now();

    // Create filter once outside the loop to avoid per-iteration WASM allocations
    const wasm = await this.#getWasm();
    const txIdObj = wasm.TransactionId.fromHex(txId);
    const filter = wasm.TransactionFilter.ids([txIdObj]);

    while (true) {
      const elapsed = Date.now() - start;
      if (timeout > 0 && elapsed >= timeout) {
        throw new Error(
          `Transaction confirmation timed out after ${timeout}ms`
        );
      }

      try {
        await this.#inner.syncStateWithTimeout(0);
      } catch {
        // Sync may fail transiently; continue polling
      }

      const txs = await this.#inner.getTransactions(filter);

      if (txs && txs.length > 0) {
        const tx = txs[0];
        const status = tx.transactionStatus?.();

        if (status) {
          if (status.isCommitted()) {
            opts?.onProgress?.("committed");
            return;
          }
          if (status.isDiscarded()) {
            throw new Error(`Transaction rejected: ${txId}`);
          }
        }

        opts?.onProgress?.("submitted");
      } else {
        opts?.onProgress?.("pending");
      }

      await new Promise((resolve) => setTimeout(resolve, interval));
    }
  }

  async #resolveNoteInput(input) {
    if (typeof input === "string") {
      const record = await this.#inner.getInputNote(input);
      if (!record) {
        throw new Error(`Note not found: ${input}`);
      }
      return record.toNote();
    }
    if (input && typeof input.toNote === "function") {
      return input.toNote();
    }
    return input;
  }

  async #submitOrSubmitWithProver(accountId, request, perCallProver) {
    const prover = perCallProver ?? this.#client.defaultProver;
    if (prover) {
      return await this.#inner.submitNewTransactionWithProver(
        accountId,
        request,
        prover
      );
    }
    return await this.#inner.submitNewTransaction(accountId, request);
  }
}
