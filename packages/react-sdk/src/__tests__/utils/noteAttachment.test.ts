import { describe, it, expect, vi } from "vitest";
import {
  readNoteAttachment,
  createNoteAttachment,
} from "../../utils/noteAttachment";
import {
  NoteAttachmentKind,
  NoteAttachment,
} from "@miden-sdk/miden-sdk";

describe("readNoteAttachment", () => {
  it("should return null when note has no metadata", () => {
    const note = { metadata: vi.fn(() => null) } as any;
    expect(readNoteAttachment(note)).toBeNull();
  });

  it("should return null when metadata has no attachment", () => {
    const note = {
      metadata: vi.fn(() => ({
        attachment: vi.fn(() => null),
      })),
    } as any;
    expect(readNoteAttachment(note)).toBeNull();
  });

  it("should return null when attachment kind is None", () => {
    const note = {
      metadata: vi.fn(() => ({
        attachment: vi.fn(() => ({
          kind: vi.fn(() => NoteAttachmentKind.None),
        })),
      })),
    } as any;
    expect(readNoteAttachment(note)).toBeNull();
  });

  it("should read Word attachment", () => {
    const note = {
      metadata: vi.fn(() => ({
        attachment: vi.fn(() => ({
          kind: vi.fn(() => NoteAttachmentKind.Word),
          asWord: vi.fn(() => ({
            toU64s: () => [1n, 2n, 3n, 4n],
          })),
        })),
      })),
    } as any;

    const result = readNoteAttachment(note);
    expect(result).toEqual({
      values: [1n, 2n, 3n, 4n],
      kind: "word",
    });
  });

  it("should read Array attachment", () => {
    const note = {
      metadata: vi.fn(() => ({
        attachment: vi.fn(() => ({
          kind: vi.fn(() => NoteAttachmentKind.Array),
          asArray: vi.fn(() => ({
            toU64s: () => [10n, 20n, 30n, 40n, 50n, 60n, 70n, 80n],
          })),
        })),
      })),
    } as any;

    const result = readNoteAttachment(note);
    expect(result).toEqual({
      values: [10n, 20n, 30n, 40n, 50n, 60n, 70n, 80n],
      kind: "array",
    });
  });

  it("should return null when asWord returns null", () => {
    const note = {
      metadata: vi.fn(() => ({
        attachment: vi.fn(() => ({
          kind: vi.fn(() => NoteAttachmentKind.Word),
          asWord: vi.fn(() => null),
        })),
      })),
    } as any;
    expect(readNoteAttachment(note)).toBeNull();
  });

  it("should return null on exceptions", () => {
    const note = {
      metadata: vi.fn(() => {
        throw new Error("boom");
      }),
    } as any;
    expect(readNoteAttachment(note)).toBeNull();
  });

  it("should convert number values to bigint", () => {
    const note = {
      metadata: vi.fn(() => ({
        attachment: vi.fn(() => ({
          kind: vi.fn(() => NoteAttachmentKind.Word),
          asWord: vi.fn(() => ({
            toU64s: () => [1, 2, 3, 4], // numbers, not bigints
          })),
        })),
      })),
    } as any;

    const result = readNoteAttachment(note);
    expect(result!.values).toEqual([1n, 2n, 3n, 4n]);
  });
});

describe("createNoteAttachment", () => {
  it("should create empty attachment for empty values", () => {
    const attachment = createNoteAttachment([]);
    expect(attachment).toBeInstanceOf(NoteAttachment);
  });

  it("should create Word attachment for 1-4 values", () => {
    // With the mock, NoteAttachment.newWord returns a NoteAttachment
    const attachment = createNoteAttachment([1n, 2n]);
    expect(attachment).toBeDefined();
  });

  it("should pad to 4 elements for Word", () => {
    // The function pads to 4 elements before creating Word
    // We verify it doesn't throw with fewer than 4 values
    expect(() => createNoteAttachment([1n])).not.toThrow();
    expect(() => createNoteAttachment([1n, 2n, 3n])).not.toThrow();
    expect(() => createNoteAttachment([1n, 2n, 3n, 4n])).not.toThrow();
  });

  it("should create Array attachment for > 4 values", () => {
    const attachment = createNoteAttachment([1n, 2n, 3n, 4n, 5n]);
    expect(attachment).toBeDefined();
  });

  it("should accept number[] input", () => {
    expect(() => createNoteAttachment([1, 2, 3])).not.toThrow();
  });

  it("should accept Uint8Array input", () => {
    expect(() => createNoteAttachment(new Uint8Array([1, 2, 3]))).not.toThrow();
  });

  it("should pad Array attachment to multiple of 4", () => {
    // 5 values → padded to 8 (2 Words)
    // 6 values → padded to 8 (2 Words)
    // 9 values → padded to 12 (3 Words)
    expect(() => createNoteAttachment([1n, 2n, 3n, 4n, 5n])).not.toThrow();
    expect(() =>
      createNoteAttachment([1n, 2n, 3n, 4n, 5n, 6n, 7n, 8n, 9n])
    ).not.toThrow();
  });
});
