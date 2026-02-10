import { resolveAccountRef, resolveNoteType } from "./utils.js";

// Module-level WASM reference, set by index.js after initialization
let _wasm = null;
let _WebClient = null;

export function _setWasm(wasm) {
  _wasm = wasm;
}

export function _setWebClient(WebClientClass) {
  _WebClient = WebClientClass;
}

function getWasm() {
  if (!_wasm) {
    throw new Error(
      "WASM not initialized. Ensure the SDK is loaded before calling standalone utilities."
    );
  }
  return _wasm;
}

/**
 * Creates a P2ID (Pay-to-ID) note.
 *
 * @param {NoteOptions} opts - Note creation options.
 * @returns {OutputNote} The created output note.
 */
export function createP2IDNote(opts) {
  const wasm = getWasm();
  const sender = resolveAccountRef(opts.from, wasm);
  const target = resolveAccountRef(opts.to, wasm);
  const noteAssets = buildNoteAssets(opts.assets, wasm);
  const noteType = resolveNoteType(opts.type, wasm);
  const attachment = opts.attachment
    ? new wasm.NoteAttachment(opts.attachment)
    : new wasm.NoteAttachment([]);

  const note = wasm.Note.createP2IDNote(
    sender,
    target,
    noteAssets,
    noteType,
    attachment
  );
  return wasm.OutputNote.full(note);
}

/**
 * Creates a P2IDE (Pay-to-ID with Expiration) note.
 *
 * @param {P2IDEOptions} opts - Note creation options with timelock/reclaim.
 * @returns {OutputNote} The created output note.
 */
export function createP2IDENote(opts) {
  const wasm = getWasm();
  const sender = resolveAccountRef(opts.from, wasm);
  const target = resolveAccountRef(opts.to, wasm);
  const noteAssets = buildNoteAssets(opts.assets, wasm);
  const noteType = resolveNoteType(opts.type, wasm);
  const attachment = opts.attachment
    ? new wasm.NoteAttachment(opts.attachment)
    : new wasm.NoteAttachment([]);

  const note = wasm.Note.createP2IDENote(
    sender,
    target,
    noteAssets,
    opts.reclaimAfter,
    opts.timelockUntil,
    noteType,
    attachment
  );
  return wasm.OutputNote.full(note);
}

/**
 * Builds a swap tag for note matching.
 *
 * @param {BuildSwapTagOptions} opts - Swap tag options.
 * @returns {number} The computed swap tag.
 */
export function buildSwapTag(opts) {
  const wasm = getWasm();
  if (!_WebClient || typeof _WebClient.buildSwapTag !== "function") {
    throw new Error(
      "WebClient.buildSwapTag is not available. Ensure the SDK is fully loaded."
    );
  }
  const noteType = resolveNoteType(opts.type, wasm);
  const offeredFaucetId = resolveAccountRef(opts.offer.token, wasm);
  const requestedFaucetId = resolveAccountRef(opts.request.token, wasm);

  return _WebClient.buildSwapTag(
    noteType,
    offeredFaucetId,
    BigInt(opts.offer.amount),
    requestedFaucetId,
    BigInt(opts.request.amount)
  );
}

function buildNoteAssets(assets, wasm) {
  const assetArray = Array.isArray(assets) ? assets : [assets];
  const fungibleAssets = assetArray.map((asset) => {
    const faucetId = resolveAccountRef(asset.token, wasm);
    return new wasm.FungibleAsset(faucetId, BigInt(asset.amount));
  });
  return new wasm.NoteAssets(fungibleAssets);
}
