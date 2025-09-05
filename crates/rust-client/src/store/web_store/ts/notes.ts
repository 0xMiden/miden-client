import {
  db,
  inputNotes,
  outputNotes,
  notesScripts,
  IInputNote,
  IOutputNote,
} from "./schema.js";

import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";

export async function getOutputNotes(states: Uint8Array) {
  try {
    let notes =
      states.length == 0
        ? await outputNotes.toArray()
        : await outputNotes.where("stateDiscriminant").anyOf(states).toArray();

    return await processOutputNotes(notes);
  } catch (err) {
    logWebStoreError(err, "Failed to get output notes");
  }
}

export async function getInputNotes(states: Uint8Array) {
  try {
    let notes;

    if (states.length === 0) {
      notes = await inputNotes.toArray();
    } else {
      notes = await inputNotes
        .where("stateDiscriminant")
        .anyOf(states)
        .toArray();
    }

    return await processInputNotes(notes);
  } catch (err) {
    logWebStoreError(err, "Failed to get input notes");
  }
}

export async function getInputNotesFromIds(noteIds: string[]) {
  try {
    let notes = await inputNotes.where("noteId").anyOf(noteIds).toArray();
    return await processInputNotes(notes);
  } catch (err) {
    logWebStoreError(err, "Failed to get input notes from IDs");
  }
}

export async function getInputNotesFromNullifiers(nullifiers: string[]) {
  try {
    let notes = await inputNotes.where("nullifier").anyOf(nullifiers).toArray();
    return await processInputNotes(notes);
  } catch (err) {
    logWebStoreError(err, "Failed to get input notes from nullifiers");
  }
}

export async function getOutputNotesFromNullifiers(nullifiers: string[]) {
  try {
    let notes = await outputNotes
      .where("nullifier")
      .anyOf(nullifiers)
      .toArray();
    return await processOutputNotes(notes);
  } catch (err) {
    logWebStoreError(err, "Failed to get output notes from nullifiers");
  }
}

export async function getOutputNotesFromIds(noteIds: string[]) {
  try {
    let notes = await outputNotes.where("noteId").anyOf(noteIds).toArray();
    return await processOutputNotes(notes);
  } catch (err) {
    logWebStoreError(err, "Failed to get output notes from IDs");
  }
}

export async function getUnspentInputNoteNullifiers() {
  try {
    const notes = await inputNotes
      .where("stateDiscriminant")
      .anyOf([2, 4, 5])
      .toArray();
    return notes.map((note) => note.nullifier);
  } catch (err) {
    logWebStoreError(err, "Failed to get unspent input note nullifiers");
  }
}

export async function upsertInputNote(
  noteId: string,
  assets: Uint8Array,
  serialNumber: Uint8Array,
  inputs: Uint8Array,
  scriptRoot: string,
  serializedNoteScript: Uint8Array,
  nullifier: string,
  serializedCreatedAt: string,
  stateDiscriminant: number,
  state: Uint8Array
) {
  return db.transaction("rw", inputNotes, notesScripts, async (tx) => {
    try {
      let assetsBlob = new Blob([new Uint8Array(assets)]);
      let serialNumberBlob = new Blob([new Uint8Array(serialNumber)]);
      let inputsBlob = new Blob([new Uint8Array(inputs)]);
      let stateBlob = new Blob([new Uint8Array(state)]);

      const data = {
        noteId,
        assets: assetsBlob,
        serialNumber: serialNumberBlob,
        inputs: inputsBlob,
        scriptRoot,
        nullifier,
        state: stateBlob,
        stateDiscriminant,
        serializedCreatedAt,
      };

      await tx.inputNotes.put(data);

      let serializedNoteScriptBlob = new Blob([
        new Uint8Array(serializedNoteScript),
      ]);
      const noteScriptData = {
        scriptRoot,
        serializedNoteScript: serializedNoteScriptBlob,
      };

      await tx.notesScripts.put(noteScriptData);
    } catch (error) {
      logWebStoreError(error, `Error inserting note: ${noteId}`);
    }
  });
}

export async function upsertOutputNote(
  noteId: string,
  assets: Uint8Array,
  recipientDigest: string,
  metadata: Uint8Array,
  nullifier: string | undefined,
  expectedHeight: number,
  stateDiscriminant: number,
  state: Uint8Array
) {
  return db.transaction("rw", outputNotes, notesScripts, async (tx) => {
    try {
      let assetsBlob = new Blob([new Uint8Array(assets)]);
      let metadataBlob = new Blob([new Uint8Array(metadata)]);
      let stateBlob = new Blob([new Uint8Array(state)]);

      const data = {
        noteId,
        assets: assetsBlob,
        recipientDigest,
        metadata: metadataBlob,
        nullifier: nullifier ? nullifier : undefined,
        expectedHeight,
        stateDiscriminant,
        state: stateBlob,
      };

      await tx.outputNotes.put(data);
    } catch (error) {
      logWebStoreError(error, `Error inserting note: ${noteId}`);
    }
  });
}

async function processInputNotes(notes: IInputNote[]) {
  return await Promise.all(
    notes.map(async (note) => {
      const assetsArrayBuffer = await note.assets.arrayBuffer();
      const assetsArray = new Uint8Array(assetsArrayBuffer);
      const assetsBase64 = uint8ArrayToBase64(assetsArray);

      const serialNumberBuffer = await note.serialNumber.arrayBuffer();
      const serialNumberArray = new Uint8Array(serialNumberBuffer);
      const serialNumberBase64 = uint8ArrayToBase64(serialNumberArray);

      const inputsBuffer = await note.inputs.arrayBuffer();
      const inputsArray = new Uint8Array(inputsBuffer);
      const inputsBase64 = uint8ArrayToBase64(inputsArray);

      let serializedNoteScriptBase64: string | undefined = undefined;
      if (note.scriptRoot) {
        let record = await notesScripts.get(note.scriptRoot);
        if (record) {
          let serializedNoteScriptArrayBuffer =
            await record.serializedNoteScript.arrayBuffer();
          const serializedNoteScriptArray = new Uint8Array(
            serializedNoteScriptArrayBuffer
          );
          serializedNoteScriptBase64 = uint8ArrayToBase64(
            serializedNoteScriptArray
          );
        }
      }

      const stateBuffer = await note.state.arrayBuffer();
      const stateArray = new Uint8Array(stateBuffer);
      const stateBase64 = uint8ArrayToBase64(stateArray);

      return {
        assets: assetsBase64,
        serialNumber: serialNumberBase64,
        inputs: inputsBase64,
        createdAt: note.serializedCreatedAt,
        serializedNoteScript: serializedNoteScriptBase64,
        state: stateBase64,
      };
    })
  );
}

async function processOutputNotes(notes: IOutputNote[]) {
  return await Promise.all(
    notes.map(async (note) => {
      const assetsArrayBuffer = await note.assets.arrayBuffer();
      const assetsArray = new Uint8Array(assetsArrayBuffer);
      const assetsBase64 = uint8ArrayToBase64(assetsArray);

      const metadataArrayBuffer = await note.metadata.arrayBuffer();
      const metadataArray = new Uint8Array(metadataArrayBuffer);
      const metadataBase64 = uint8ArrayToBase64(metadataArray);

      const stateBuffer = await note.state.arrayBuffer();
      const stateArray = new Uint8Array(stateBuffer);
      const stateBase64 = uint8ArrayToBase64(stateArray);

      return {
        assets: assetsBase64,
        recipientDigest: note.recipientDigest,
        metadata: metadataBase64,
        expectedHeight: note.expectedHeight,
        state: stateBase64,
      };
    })
  );
}
