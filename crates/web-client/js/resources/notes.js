import { resolveAccountRef, resolveAddress, resolveNoteIdHex } from "../utils.js";

export class NotesResource {
  #inner;
  #getWasm;
  #client;

  constructor(inner, getWasm, client) {
    this.#inner = inner;
    this.#getWasm = getWasm;
    this.#client = client;
  }

  async list(query) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();
    const filter = buildNoteFilter(query, wasm);
    return await this.#inner.getInputNotes(filter);
  }

  async get(noteId) {
    this.#client.assertNotTerminated();
    const result = await this.#inner.getInputNote(resolveNoteIdHex(noteId));
    return result ?? null;
  }

  async listSent(query) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();
    const filter = buildNoteFilter(query, wasm);
    return await this.#inner.getOutputNotes(filter);
  }

  async listAvailable(opts) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();
    const accountId = resolveAccountRef(opts.account, wasm);
    return await this.#inner.getConsumableNotes(accountId);
  }

  async import(noteFile) {
    this.#client.assertNotTerminated();
    return await this.#inner.importNoteFile(noteFile);
  }

  async export(noteId, opts) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();
    const format = opts?.format ?? wasm.NoteExportFormat.Full;
    return await this.#inner.exportNoteFile(resolveNoteIdHex(noteId), format);
  }

  async fetchPrivate(opts) {
    this.#client.assertNotTerminated();
    if (opts?.mode === "all") {
      await this.#inner.fetchAllPrivateNotes();
    } else {
      await this.#inner.fetchPrivateNotes();
    }
  }

  async sendPrivate(opts) {
    this.#client.assertNotTerminated();
    const wasm = await this.#getWasm();
    const noteHex = resolveNoteIdHex(opts.noteId);
    const noteRecord = await this.#inner.getInputNote(noteHex);
    if (!noteRecord) {
      throw new Error(`Note not found: ${noteHex}`);
    }
    const note = noteRecord.toNote();
    const address = resolveAddress(opts.to, wasm);
    await this.#inner.sendPrivateNote(note, address);
  }
}

function buildNoteFilter(query, wasm) {
  if (!query) {
    return new wasm.NoteFilter(wasm.NoteFilterTypes.All, undefined);
  }

  if (query.ids) {
    const noteIds = query.ids.map((id) => wasm.NoteId.fromHex(resolveNoteIdHex(id)));
    return new wasm.NoteFilter(wasm.NoteFilterTypes.List, noteIds);
  }

  if (query.status) {
    const statusMap = {
      consumed: wasm.NoteFilterTypes.Consumed,
      committed: wasm.NoteFilterTypes.Committed,
      expected: wasm.NoteFilterTypes.Expected,
      processing: wasm.NoteFilterTypes.Processing,
      unverified: wasm.NoteFilterTypes.Unverified,
    };
    const filterType = statusMap[query.status];
    if (filterType === undefined) {
      throw new Error(`Unknown note status: ${query.status}`);
    }
    return new wasm.NoteFilter(filterType, undefined);
  }

  return new wasm.NoteFilter(wasm.NoteFilterTypes.All, undefined);
}
