import { getDatabase, IInputNote, IOutputNote } from "./schema.js";

import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";

export async function getOutputNotes(dbId: string, states: Uint8Array) {
  try {
    const db = getDatabase(dbId);
    let notes =
      states.length == 0
        ? await db.outputNotes.toArray()
        : await db.outputNotes
            .where("stateDiscriminant")
            .anyOf(states)
            .toArray();

    return await processOutputNotes(notes);
  } catch (err) {
    logWebStoreError(err, "Failed to get output notes");
  }
}

export async function getInputNotes(dbId: string, states: Uint8Array) {
  try {
    const db = getDatabase(dbId);
    let notes;

    if (states.length === 0) {
      notes = await db.inputNotes.toArray();
    } else {
      notes = await db.inputNotes
        .where("stateDiscriminant")
        .anyOf(states)
        .toArray();
    }

    return await processInputNotes(dbId, notes);
  } catch (err) {
    logWebStoreError(err, "Failed to get input notes");
  }
}

export async function getInputNotesFromIds(dbId: string, noteIds: string[]) {
  try {
    const db = getDatabase(dbId);
    let notes = await db.inputNotes.where("noteId").anyOf(noteIds).toArray();
    return await processInputNotes(dbId, notes);
  } catch (err) {
    logWebStoreError(err, "Failed to get input notes from IDs");
  }
}

export async function getInputNotesFromNullifiers(
  dbId: string,
  nullifiers: string[]
) {
  try {
    const db = getDatabase(dbId);
    let notes = await db.inputNotes
      .where("nullifier")
      .anyOf(nullifiers)
      .toArray();
    return await processInputNotes(dbId, notes);
  } catch (err) {
    logWebStoreError(err, "Failed to get input notes from nullifiers");
  }
}

export async function getOutputNotesFromNullifiers(
  dbId: string,
  nullifiers: string[]
) {
  try {
    const db = getDatabase(dbId);
    let notes = await db.outputNotes
      .where("nullifier")
      .anyOf(nullifiers)
      .toArray();
    return await processOutputNotes(notes);
  } catch (err) {
    logWebStoreError(err, "Failed to get output notes from nullifiers");
  }
}

export async function getOutputNotesFromIds(dbId: string, noteIds: string[]) {
  try {
    const db = getDatabase(dbId);
    let notes = await db.outputNotes.where("noteId").anyOf(noteIds).toArray();
    return await processOutputNotes(notes);
  } catch (err) {
    logWebStoreError(err, "Failed to get output notes from IDs");
  }
}

export async function getUnspentInputNoteNullifiers(dbId: string) {
  try {
    const db = getDatabase(dbId);
    const notes = await db.inputNotes
      .where("stateDiscriminant")
      .anyOf([2, 4, 5])
      .toArray();
    return notes.map((note) => note.nullifier);
  } catch (err) {
    logWebStoreError(err, "Failed to get unspent input note nullifiers");
  }
}

export async function getNoteScript(dbId: string, scriptRoot: string) {
  try {
    const db = getDatabase(dbId);
    const noteScript = await db.notesScripts
      .where("scriptRoot")
      .equals(scriptRoot)
      .first();
    return noteScript;
  } catch (err) {
    logWebStoreError(err, "Failed to get note script from root");
  }
}

export async function upsertInputNote(
  dbId: string,
  noteId: string,
  assets: Uint8Array,
  serialNumber: Uint8Array,
  inputs: Uint8Array,
  scriptRoot: string,
  serializedNoteScript: Uint8Array,
  nullifier: string,
  serializedCreatedAt: string,
  stateDiscriminant: number,
  state: Uint8Array,
  consumedBlockHeight?: number | null,
  consumedTxOrder?: number | null,
  consumerAccountId?: string | null
) {
  const db = getDatabase(dbId);
  return db.dexie.transaction(
    "rw",
    db.inputNotes,
    db.notesScripts,
    async (tx) => {
      try {
        const data = {
          noteId,
          assets,
          serialNumber,
          inputs,
          scriptRoot,
          nullifier,
          state,
          stateDiscriminant,
          serializedCreatedAt,
          // Convert null -> undefined so Dexie doesn't index missing values
          consumedBlockHeight: consumedBlockHeight ?? undefined,
          consumedTxOrder: consumedTxOrder ?? undefined,
          consumerAccountId: consumerAccountId ?? undefined,
        };

        await tx.inputNotes.put(data);

        const noteScriptData = {
          scriptRoot,
          serializedNoteScript,
        };

        await tx.notesScripts.put(noteScriptData);
      } catch (error) {
        logWebStoreError(error, `Error inserting note: ${noteId}`);
      }
    }
  );
}

// When a consumer is set, uses the [consumedBlockHeight+consumedTxOrder+noteId] compound
// index for cursor-based iteration.
export async function getInputNoteByOffset(
  dbId: string,
  states: Uint8Array,
  consumerAccountId: string | undefined,
  blockStart: number | undefined,
  blockEnd: number | undefined,
  offset: number
) {
  try {
    const db = getDatabase(dbId);

    if (consumerAccountId != null) {
      // Cursor-based path: iterate the compound index in order, filter lazily.
      const results = await db.inputNotes
        .orderBy("[consumedBlockHeight+consumedTxOrder+noteId]")
        .filter((n: IInputNote) => {
          if (states.length > 0 && !states.includes(n.stateDiscriminant))
            return false;
          if (n.consumerAccountId !== consumerAccountId) return false;
          if (
            blockStart != null &&
            (n.consumedBlockHeight == null ||
              n.consumedBlockHeight < blockStart)
          )
            return false;
          if (
            blockEnd != null &&
            (n.consumedBlockHeight == null || n.consumedBlockHeight > blockEnd)
          )
            return false;
          return true;
        })
        .offset(offset)
        .limit(1)
        .toArray();

      if (results.length === 0) return [];
      return await processInputNotes(dbId, results);
    }

    // Fallback: load all matching notes, sort in memory, pick one.
    let notes: IInputNote[];
    if (states.length === 0) {
      notes = await db.inputNotes.toArray();
    } else {
      notes = await db.inputNotes
        .where("stateDiscriminant")
        .anyOf(states)
        .toArray();
    }

    if (blockStart != null) {
      notes = notes.filter(
        (n) =>
          n.consumedBlockHeight != null && n.consumedBlockHeight >= blockStart
      );
    }
    if (blockEnd != null) {
      notes = notes.filter(
        (n) =>
          n.consumedBlockHeight != null && n.consumedBlockHeight <= blockEnd
      );
    }

    notes.sort((a, b) => {
      const aH = a.consumedBlockHeight;
      const bH = b.consumedBlockHeight;
      if (aH == null && bH != null) return 1;
      if (aH != null && bH == null) return -1;
      if (aH != null && bH != null && aH !== bH) return aH - bH;
      const aO = a.consumedTxOrder;
      const bO = b.consumedTxOrder;
      if (aO == null && bO != null) return 1;
      if (aO != null && bO == null) return -1;
      if (aO != null && bO != null && aO !== bO) return aO - bO;
      if (a.noteId < b.noteId) return -1;
      if (a.noteId > b.noteId) return 1;
      return 0;
    });

    const note = notes[offset];
    if (!note) return [];
    return await processInputNotes(dbId, [note]);
  } catch (err) {
    logWebStoreError(err, "Failed to get input note by offset");
  }
}

export async function upsertOutputNote(
  dbId: string,
  noteId: string,
  assets: Uint8Array,
  recipientDigest: string,
  metadata: Uint8Array,
  nullifier: string | undefined,
  expectedHeight: number,
  stateDiscriminant: number,
  state: Uint8Array
) {
  const db = getDatabase(dbId);
  return db.dexie.transaction(
    "rw",
    db.outputNotes,
    db.notesScripts,
    async (tx) => {
      try {
        const data = {
          noteId,
          assets,
          recipientDigest,
          metadata,
          nullifier: nullifier ? nullifier : undefined,
          expectedHeight,
          stateDiscriminant,
          state,
        };

        await tx.outputNotes.put(data);
      } catch (error) {
        logWebStoreError(error, `Error inserting note: ${noteId}`);
      }
    }
  );
}

async function processInputNotes(dbId: string, notes: IInputNote[]) {
  const db = getDatabase(dbId);
  return await Promise.all(
    notes.map(async (note) => {
      const assetsBase64 = uint8ArrayToBase64(note.assets);

      const serialNumberBase64 = uint8ArrayToBase64(note.serialNumber);

      const inputsBase64 = uint8ArrayToBase64(note.inputs);

      let serializedNoteScriptBase64: string | undefined = undefined;
      if (note.scriptRoot) {
        let record = await db.notesScripts.get(note.scriptRoot);
        if (record) {
          serializedNoteScriptBase64 = uint8ArrayToBase64(
            record.serializedNoteScript
          );
        }
      }

      const stateBase64 = uint8ArrayToBase64(note.state);

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
    notes.map((note) => {
      const assetsBase64 = uint8ArrayToBase64(note.assets);

      const metadataBase64 = uint8ArrayToBase64(note.metadata);

      const stateBase64 = uint8ArrayToBase64(note.state);

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

export async function upsertNoteScript(
  dbId: string,
  scriptRoot: string,
  serializedNoteScript: Uint8Array
) {
  const db = getDatabase(dbId);
  return db.dexie.transaction(
    "rw",
    db.outputNotes,
    db.notesScripts,
    async (tx) => {
      try {
        const noteScriptData = {
          scriptRoot,
          serializedNoteScript,
        };

        await tx.notesScripts.put(noteScriptData);
      } catch (error) {
        logWebStoreError(error, `Error inserting note script: ${scriptRoot}`);
      }
    }
  );
}
