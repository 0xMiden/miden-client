import {
  resolveAccountRef,
  resolveNoteType,
  resolveTransactionIdHex,
} from "../utils.js";

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

    if (opts.authenticated === false) {
      // Unauthenticated P2ID path — create note in JS, return Note object
      if (opts.reclaimAfter != null || opts.timelockUntil != null) {
        throw new Error(
          "reclaimAfter and timelockUntil are not supported for unauthenticated sends"
        );
      }

      const senderId = resolveAccountRef(opts.account, wasm);
      const receiverId = resolveAccountRef(opts.to, wasm);
      const faucetId = resolveAccountRef(opts.token, wasm);
      const noteType = resolveNoteType(opts.type, wasm);

      const note = wasm.Note.createP2IDNote(
        senderId,
        receiverId,
        new wasm.NoteAssets([
          new wasm.FungibleAsset(faucetId, BigInt(opts.amount)),
        ]),
        noteType,
        new wasm.NoteAttachment()
      );

      const request = new wasm.TransactionRequestBuilder()
        .withOwnOutputNotes(
          new wasm.OutputNoteArray([wasm.OutputNote.full(note)])
        )
        .build();

      const txId = await this.#submitOrSubmitWithProver(
        senderId,
        request,
        opts.prover
      );

      if (opts.waitForConfirmation) {
        await this.waitFor(txId.toHex(), { timeout: opts.timeout });
      }

      return { txId, note };
    }

    // Authenticated path (default) — existing P2ID with optional reclaim/timelock
    const { accountId, request } = await this.#buildSendRequest(opts, wasm);
    const txId = await this.#submitOrSubmitWithProver(
      accountId,
      request,
      opts.prover
    );

    if (opts.waitForConfirmation) {
      await this.waitFor(txId.toHex(), { timeout: opts.timeout });
    }

    return { txId, note: null };
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

  async execute(opts) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();
    const accountId = resolveAccountRef(opts.account, wasm);

    let builder = new wasm.TransactionRequestBuilder().withCustomScript(
      opts.script
    );

    if (opts.foreignAccounts?.length) {
      const accounts = opts.foreignAccounts.map((fa) => {
        // Distinguish { id: AccountRef, storage? } wrapper objects from WASM types
        // (Account/AccountHeader expose .id() as a method, wrappers have .id as a property)
        const isWrapper =
          fa !== null &&
          typeof fa === "object" &&
          "id" in fa &&
          typeof fa.id !== "function";
        const id = resolveAccountRef(isWrapper ? fa.id : fa, wasm);
        const storage =
          isWrapper && fa.storage
            ? fa.storage
            : new wasm.AccountStorageRequirements();
        return wasm.ForeignAccount.public(id, storage);
      });
      builder = builder.withForeignAccounts(
        new wasm.ForeignAccountArray(accounts)
      );
    }

    const request = builder.build();
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
      const txIds = query.ids.map((id) =>
        wasm.TransactionId.fromHex(resolveTransactionIdHex(id))
      );
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
   * @param {string | TransactionId} txId - Transaction ID hex string or TransactionId object.
   * @param {WaitOptions} [opts] - Polling options.
   * @param {number} [opts.timeout=60000] - Wall-clock polling timeout in
   *   milliseconds. This is NOT a block height — it controls how long the
   *   client waits before giving up. Set to 0 to disable the timeout and poll
   *   indefinitely until the transaction is committed or discarded.
   * @param {number} [opts.interval=5000] - Polling interval in ms.
   * @param {function} [opts.onProgress] - Called with the current status on
   *   each poll iteration ("pending", "submitted", or "committed").
   */
  async waitFor(txId, opts) {
    this.#client.assertNotTerminated();
    const hex = resolveTransactionIdHex(txId);
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
        wasm.TransactionId.fromHex(hex),
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
            throw new Error(`Transaction rejected: ${hex}`);
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

    // Detect direct Note objects (unauthenticated consumption path).
    // Note objects have .id() but not .toNote(); NoteId has neither .id() nor .toNote().
    const hasDirectNotes = noteInputs.some(
      (input) =>
        input !== null &&
        typeof input === "object" &&
        typeof input.id === "function" &&
        typeof input.toNote !== "function"
    );

    if (hasDirectNotes) {
      // Resolve all inputs to Note objects, then use NoteAndArgs builder path
      // (the only WASM path that accepts unauthenticated notes not in the store).
      const resolvedNotes = await Promise.all(
        noteInputs.map(async (input) => {
          // Already a Note object
          if (
            input !== null &&
            typeof input === "object" &&
            typeof input.id === "function" &&
            typeof input.toNote !== "function"
          ) {
            return input;
          }
          // InputNoteRecord
          if (input && typeof input.toNote === "function") {
            return input.toNote();
          }
          // String or NoteId — fetch from store
          return await this.#resolveNoteInput(input);
        })
      );

      const noteAndArgsArr = resolvedNotes.map(
        (note) => new wasm.NoteAndArgs(note, null)
      );
      const request = new wasm.TransactionRequestBuilder()
        .withInputNotes(new wasm.NoteAndArgsArray(noteAndArgsArr))
        .build();
      return { accountId, request };
    }

    // Standard path: all inputs are IDs or records — look up from store.
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
    const paybackNoteType = resolveNoteType(
      opts.paybackType ?? opts.type,
      wasm
    );

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
    // NoteId — has toString() but not toNote() or id() (unlike InputNoteRecord/Note).
    // Check for constructor.fromHex to distinguish from plain objects.
    if (
      input &&
      typeof input.toString === "function" &&
      typeof input.toNote !== "function" &&
      typeof input.id !== "function" &&
      input.constructor?.fromHex !== undefined
    ) {
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
