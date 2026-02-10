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
    const { accountId, request } = await this.#buildSendRequest(opts, wasm);

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
    const { accountId, request } = await this.#buildMintRequest(opts, wasm);

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
    const { accountId, request } = await this.#buildConsumeRequest(opts, wasm);

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

    // getConsumableNotes takes AccountId by value (consumed by WASM).
    // Save hex so we can reconstruct for submitNewTransaction.
    const accountId = resolveAccountRef(opts.account, wasm);
    const accountIdHex = accountId.toString();
    const consumable = await this.#inner.getConsumableNotes(accountId);

    if (!consumable || consumable.length === 0) {
      return { txId: null, consumed: 0, remaining: 0 };
    }

    const total = consumable.length;
    const toConsume =
      opts.maxNotes != null ? consumable.slice(0, opts.maxNotes) : consumable;

    if (toConsume.length === 0) {
      return { txId: null, consumed: 0, remaining: total };
    }

    const notes = toConsume.map((c) => c.inputNoteRecord().toNote());

    const request = await this.#inner.newConsumeTransactionRequest(notes);

    const txId = await this.#submitOrSubmitWithProver(
      wasm.AccountId.fromHex(accountIdHex),
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
    const { accountId, request } = await this.#buildSwapRequest(opts, wasm);

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

    // Save hex strings upfront — getConsumableNotes takes AccountId by value,
    // which consumes the JS object. We reconstruct from hex when needed.
    const faucetId = resolveAccountRef(opts.faucet, wasm);
    const targetId = resolveAccountRef(opts.to, wasm);
    const faucetIdHex = faucetId.toString();
    const targetIdHex = targetId.toString();
    const noteType = resolveNoteType(opts.type, wasm);
    const amount = BigInt(opts.amount);

    // Capture existing consumable note IDs before mint to avoid consuming
    // pre-existing notes that don't belong to this operation.
    // getConsumableNotes consumes targetId by value.
    const preMintConsumable = await this.#inner.getConsumableNotes(targetId);
    const preExistingNoteIds = new Set(
      preMintConsumable.map((c) => c.inputNoteRecord().toNote().id().toString())
    );

    // Step 1: Mint
    // newMintTransactionRequest borrows (&AccountId), but submitNewTransaction
    // serializes via .toString() through the worker — needs a fresh ID each time.
    let mintTxId;
    try {
      const mintRequest = await this.#inner.newMintTransactionRequest(
        wasm.AccountId.fromHex(targetIdHex),
        wasm.AccountId.fromHex(faucetIdHex),
        noteType,
        amount
      );
      mintTxId = await this.#submitOrSubmitWithProver(
        wasm.AccountId.fromHex(faucetIdHex),
        mintRequest,
        opts.prover
      );
    } catch (original) {
      const err = new Error(original.message);
      err.step = "mint";
      err.cause = original;
      throw err;
    }

    // Step 2: Wait for mint confirmation (always required — can't consume before mint confirms)
    try {
      await this.waitFor(mintTxId.toHex(), { timeout: opts.timeout });
    } catch (original) {
      const err = new Error(original.message);
      err.step = "sync";
      err.cause = original;
      throw err;
    }

    // Step 3: Consume only the newly minted notes.
    // The minted note may not appear in getConsumableNotes immediately after
    // waitFor returns (the transaction is confirmed but the note may not yet
    // be indexed locally). Retry with sync up to 5 times before giving up.
    try {
      const MAX_CONSUME_ATTEMPTS = 6;
      const CONSUME_RETRY_INTERVAL = 3_000;
      let newNotes = [];

      for (let attempt = 0; attempt < MAX_CONSUME_ATTEMPTS; attempt++) {
        // getConsumableNotes consumes AccountId by value — use fresh ID
        const consumable = await this.#inner.getConsumableNotes(
          wasm.AccountId.fromHex(targetIdHex)
        );
        newNotes = consumable
          .filter(
            (c) =>
              !preExistingNoteIds.has(c.inputNoteRecord().toNote().id().toString())
          )
          .map((c) => c.inputNoteRecord().toNote());

        if (newNotes.length > 0) break;

        if (attempt < MAX_CONSUME_ATTEMPTS - 1) {
          await new Promise((r) => setTimeout(r, CONSUME_RETRY_INTERVAL));
          try {
            await this.#inner.syncStateWithTimeout(0);
          } catch {
            // Sync may fail transiently; continue retrying
          }
        }
      }

      if (newNotes.length === 0) {
        throw new Error(
          "No consumable notes found after mint (retries exhausted)"
        );
      }

      const consumeRequest =
        await this.#inner.newConsumeTransactionRequest(newNotes);
      const consumeTxId = await this.#submitOrSubmitWithProver(
        wasm.AccountId.fromHex(targetIdHex),
        consumeRequest,
        opts.prover
      );

      if (opts.waitForConfirmation) {
        await this.waitFor(consumeTxId.toHex(), { timeout: opts.timeout });
      }
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
        ({ accountId, request } = await this.#buildSendRequest(opts, wasm));
        break;
      }
      case "mint": {
        ({ accountId, request } = await this.#buildMintRequest(opts, wasm));
        break;
      }
      case "consume": {
        ({ accountId, request } = await this.#buildConsumeRequest(opts, wasm));
        break;
      }
      case "swap": {
        ({ accountId, request } = await this.#buildSwapRequest(opts, wasm));
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

  /**
   * Polls for transaction confirmation.
   *
   * @param {string} txId - Transaction ID hex string.
   * @param {WaitOptions} [opts] - Polling options.
   * @param {number} [opts.timeout=60000] - Timeout in ms. Set to 0 to disable
   *   the timeout and poll indefinitely until the transaction is committed or
   *   discarded.
   * @param {number} [opts.interval=5000] - Polling interval in ms.
   * @param {function} [opts.onProgress] - Called with the current status on
   *   each poll iteration ("pending", "submitted", or "committed").
   */
  async waitFor(txId, opts) {
    this.#client.assertNotTerminated();
    const timeout = opts?.timeout ?? 60_000;
    const interval = opts?.interval ?? 5_000;
    const start = Date.now();

    const wasm = await this.#getWasm();

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

      // Recreate filter each iteration — WASM consumes it by value
      const filter = wasm.TransactionFilter.ids([
        wasm.TransactionId.fromHex(txId),
      ]);
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

  // ── Shared request builders ──

  async #buildSendRequest(opts, wasm) {
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
    return { accountId, request };
  }

  async #buildMintRequest(opts, wasm) {
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
    return { accountId, request };
  }

  async #buildConsumeRequest(opts, wasm) {
    const accountId = resolveAccountRef(opts.account, wasm);
    const noteInputs = Array.isArray(opts.notes) ? opts.notes : [opts.notes];
    const notes = await Promise.all(
      noteInputs.map((input) => this.#resolveNoteInput(input))
    );
    const request = await this.#inner.newConsumeTransactionRequest(notes);
    return { accountId, request };
  }

  async #buildSwapRequest(opts, wasm) {
    const accountId = resolveAccountRef(opts.account, wasm);
    const offeredFaucetId = resolveAccountRef(opts.offer.token, wasm);
    const requestedFaucetId = resolveAccountRef(opts.request.token, wasm);
    const noteType = resolveNoteType(opts.type, wasm);
    const paybackNoteType = resolveNoteType(opts.paybackType ?? opts.type, wasm);

    const request = await this.#inner.newSwapTransactionRequest(
      accountId,
      offeredFaucetId,
      BigInt(opts.offer.amount),
      requestedFaucetId,
      BigInt(opts.request.amount),
      noteType,
      paybackNoteType
    );
    return { accountId, request };
  }

  async #resolveNoteInput(input) {
    if (typeof input === "string") {
      const record = await this.#inner.getInputNote(input);
      if (!record) {
        throw new Error(`Note not found: ${input}`);
      }
      return record.toNote();
    }
    // InputNoteRecord — unwrap to Note
    if (input && typeof input.toNote === "function") {
      return input.toNote();
    }
    // NoteId — look up the note by its hex ID
    if (input && input.constructor?.name === "NoteId") {
      const hex = input.toString();
      const record = await this.#inner.getInputNote(hex);
      if (!record) {
        throw new Error(`Note not found: ${hex}`);
      }
      return record.toNote();
    }
    // Assume it's already a Note object
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
