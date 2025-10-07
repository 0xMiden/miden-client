import test from "./playwright.global.setup";
import { Page, expect } from "@playwright/test";
import {
  clearStore,
  getInputNote,
  setupMintedNote,
} from "./webClientTestUtils";

const exportNoteFile = async (page: Page, noteId: string) => {
  return await page.evaluate(async (_noteId) => {
    const client = window.client;
    const noteFile = await client.exportNoteFile(_noteId, "Details");
    return Array.from(noteFile.serialize());
  }, noteId);
};

const importNoteFile = async (page: Page, noteBytes: number[]) => {
  return await page.evaluate(async (_noteBytes) => {
    const client = window.client;
    const noteFile = window.NoteFile.deserialize(new Uint8Array(_noteBytes));
    return await client.importNoteFile(noteFile);
  }, noteBytes);
};

test.describe("NoteFile", () => {
  test("it serializes and deserializes a note file", async ({ page }) => {
    const { createdNoteId } = await setupMintedNote(page);

    const exportedBytes = await exportNoteFile(page, createdNoteId);

    const reserializedBytes = await page.evaluate(async (_bytes) => {
      const byteArray = new Uint8Array(_bytes);
      const noteFile = window.NoteFile.deserialize(byteArray);
      return Array.from(noteFile.serialize());
    }, exportedBytes);

    expect(reserializedBytes).toEqual(exportedBytes);

    await clearStore(page);

    const importedNoteId = await importNoteFile(page, reserializedBytes);
    expect(importedNoteId).toBe(createdNoteId);

    const { noteId } = await getInputNote(createdNoteId, page);
    expect(noteId).toBe(createdNoteId);
  });
});
