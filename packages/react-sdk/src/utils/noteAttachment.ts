import {
  NoteAttachment,
  NoteAttachmentKind,
  NoteAttachmentScheme,
  Word,
} from "@miden-sdk/miden-sdk";
import type { InputNoteRecord } from "@miden-sdk/miden-sdk";

export interface NoteAttachmentData {
  values: bigint[];
  kind: "word" | "array";
}

// Runtime WASM objects may have methods not in the TS declarations.
// Use a loose shape for runtime-only properties.
type NoteMetadataRuntime = {
  attachment?: () => {
    kind?: () => unknown;
    asWord?: () => { toU64s: () => Iterable<unknown> } | null;
    asArray?: () => { toU64s: () => Iterable<unknown> } | null;
  } | null;
};

/**
 * Decode a note's attachment. Returns null if no attachment.
 */
export function readNoteAttachment(
  note: InputNoteRecord
): NoteAttachmentData | null {
  try {
    const metadata = note.metadata?.() as NoteMetadataRuntime | null;
    if (!metadata) return null;

    const attachment = metadata.attachment?.();
    if (!attachment) return null;

    const kind = attachment.kind?.();
    if (!kind) return null;

    if (kind === NoteAttachmentKind.None) return null;

    if (kind === NoteAttachmentKind.Word) {
      const word = attachment.asWord?.();
      if (!word) return null;
      const u64s = word.toU64s();
      const values = Array.from(u64s as Iterable<unknown>).map((v) =>
        BigInt(v as number | bigint)
      );
      return { values, kind: "word" };
    }

    if (kind === NoteAttachmentKind.Array) {
      const arr = attachment.asArray?.();
      if (!arr) return null;
      const u64s = arr.toU64s();
      const values = Array.from(u64s as Iterable<unknown>).map((v) =>
        BigInt(v as number | bigint)
      );
      return { values, kind: "array" };
    }

    return null;
  } catch {
    return null;
  }
}

/**
 * Encode values into a NoteAttachment.
 * <= 4 values -> Word (avoids miden-standards 0.13.x Array advice-map bug).
 * > 4 values -> Array.
 *
 * Note: Values are padded to word boundaries (multiples of 4) with trailing 0n.
 * `readNoteAttachment` returns raw values including padding. Consumers should
 * strip trailing zeros if they need the original unpadded values.
 */
export function createNoteAttachment(
  values: bigint[] | Uint8Array | number[]
): NoteAttachment {
  // Convert all values to bigint
  const bigints: bigint[] = [];
  for (let i = 0; i < values.length; i++) {
    bigints.push(BigInt(values[i]));
  }

  if (bigints.length === 0) {
    return new NoteAttachment();
  }

  const scheme = NoteAttachmentScheme.none();

  if (bigints.length <= 4) {
    // Pad to 4 elements for Word
    while (bigints.length < 4) {
      bigints.push(0n);
    }
    const word = new Word(BigUint64Array.from(bigints));
    return NoteAttachment.newWord(scheme, word);
  }

  // For > 4 values, use Array attachment
  // Pad to multiple of 4 for Word alignment
  while (bigints.length % 4 !== 0) {
    bigints.push(0n);
  }
  const words: Word[] = [];
  for (let i = 0; i < bigints.length; i += 4) {
    words.push(new Word(BigUint64Array.from(bigints.slice(i, i + 4))));
  }
  // NoteAttachment.newArray exists in the WASM bindings but is not yet
  // exposed in the TS declarations (added in SDK ≥0.13.1). Access via
  // bracket notation with a runtime guard until upstream types are updated.
  const newArray = (NoteAttachment as unknown as Record<string, unknown>)[
    "newArray"
  ];
  if (typeof newArray !== "function") {
    throw new Error(
      "NoteAttachment.newArray is not available. Ensure @miden-sdk/miden-sdk >= 0.13.1."
    );
  }
  return newArray.call(NoteAttachment, scheme, words) as NoteAttachment;
}
