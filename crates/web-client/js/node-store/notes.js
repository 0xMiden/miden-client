import { getDatabase } from "./schema.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";

export async function getOutputNotes(dbId, states) {
  try {
    const db = getDatabase(dbId);
    const statesArr = Array.from(states);
    let rows;
    if (statesArr.length === 0) {
      rows = db.prepare("SELECT * FROM output_notes").all();
    } else {
      const placeholders = statesArr.map(() => "?").join(",");
      rows = db
        .prepare(
          `SELECT * FROM output_notes WHERE stateDiscriminant IN (${placeholders})`
        )
        .all(...statesArr);
    }
    return processOutputNotes(rows);
  } catch (err) {
    logWebStoreError(err, "Failed to get output notes");
  }
}

export async function getInputNotes(dbId, states) {
  try {
    const db = getDatabase(dbId);
    const statesArr = Array.from(states);
    let rows;
    if (statesArr.length === 0) {
      rows = db.prepare("SELECT * FROM input_notes").all();
    } else {
      const placeholders = statesArr.map(() => "?").join(",");
      rows = db
        .prepare(
          `SELECT * FROM input_notes WHERE stateDiscriminant IN (${placeholders})`
        )
        .all(...statesArr);
    }
    return processInputNotes(db, rows);
  } catch (err) {
    logWebStoreError(err, "Failed to get input notes");
  }
}

export async function getInputNotesFromIds(dbId, noteIds) {
  try {
    const db = getDatabase(dbId);
    const ids = Array.from(noteIds);
    if (ids.length === 0) return [];
    const placeholders = ids.map(() => "?").join(",");
    const rows = db
      .prepare(`SELECT * FROM input_notes WHERE noteId IN (${placeholders})`)
      .all(...ids);
    return processInputNotes(db, rows);
  } catch (err) {
    logWebStoreError(err, "Failed to get input notes from IDs");
  }
}

export async function getInputNotesFromNullifiers(dbId, nullifiers) {
  try {
    const db = getDatabase(dbId);
    const nulls = Array.from(nullifiers);
    if (nulls.length === 0) return [];
    const placeholders = nulls.map(() => "?").join(",");
    const rows = db
      .prepare(`SELECT * FROM input_notes WHERE nullifier IN (${placeholders})`)
      .all(...nulls);
    return processInputNotes(db, rows);
  } catch (err) {
    logWebStoreError(err, "Failed to get input notes from nullifiers");
  }
}

export async function getOutputNotesFromNullifiers(dbId, nullifiers) {
  try {
    const db = getDatabase(dbId);
    const nulls = Array.from(nullifiers);
    if (nulls.length === 0) return [];
    const placeholders = nulls.map(() => "?").join(",");
    const rows = db
      .prepare(
        `SELECT * FROM output_notes WHERE nullifier IN (${placeholders})`
      )
      .all(...nulls);
    return processOutputNotes(rows);
  } catch (err) {
    logWebStoreError(err, "Failed to get output notes from nullifiers");
  }
}

export async function getOutputNotesFromIds(dbId, noteIds) {
  try {
    const db = getDatabase(dbId);
    const ids = Array.from(noteIds);
    if (ids.length === 0) return [];
    const placeholders = ids.map(() => "?").join(",");
    const rows = db
      .prepare(`SELECT * FROM output_notes WHERE noteId IN (${placeholders})`)
      .all(...ids);
    return processOutputNotes(rows);
  } catch (err) {
    logWebStoreError(err, "Failed to get output notes from IDs");
  }
}

export async function getUnspentInputNoteNullifiers(dbId) {
  try {
    const db = getDatabase(dbId);
    const rows = db
      .prepare(
        "SELECT nullifier FROM input_notes WHERE stateDiscriminant IN (2, 4, 5)"
      )
      .all();
    return rows.map((r) => r.nullifier);
  } catch (err) {
    logWebStoreError(err, "Failed to get unspent input note nullifiers");
  }
}

export async function getNoteScript(dbId, scriptRoot) {
  try {
    const db = getDatabase(dbId);
    const record = db
      .prepare("SELECT * FROM notes_scripts WHERE scriptRoot = ?")
      .get(scriptRoot);
    return record || undefined;
  } catch (err) {
    logWebStoreError(err, "Failed to get note script from root");
  }
}

export async function upsertInputNote(
  dbId,
  noteId,
  assets,
  serialNumber,
  inputs,
  scriptRoot,
  serializedNoteScript,
  nullifier,
  serializedCreatedAt,
  stateDiscriminant,
  state
) {
  const db = getDatabase(dbId);
  try {
    const run = db.transaction(() => {
      db.prepare(
        `INSERT OR REPLACE INTO input_notes
         (noteId, assets, serialNumber, inputs, scriptRoot, nullifier, stateDiscriminant, state, serializedCreatedAt)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)`
      ).run(
        noteId,
        assets,
        serialNumber,
        inputs,
        scriptRoot,
        nullifier,
        stateDiscriminant,
        state,
        serializedCreatedAt
      );
      db.prepare(
        "INSERT OR REPLACE INTO notes_scripts (scriptRoot, serializedNoteScript) VALUES (?, ?)"
      ).run(scriptRoot, serializedNoteScript);
    });
    run();
  } catch (error) {
    logWebStoreError(error, `Error inserting note: ${noteId}`);
  }
}

export async function upsertOutputNote(
  dbId,
  noteId,
  assets,
  recipientDigest,
  metadata,
  nullifier,
  expectedHeight,
  stateDiscriminant,
  state
) {
  const db = getDatabase(dbId);
  try {
    db.prepare(
      `INSERT OR REPLACE INTO output_notes
       (noteId, assets, recipientDigest, metadata, nullifier, expectedHeight, stateDiscriminant, state)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?)`
    ).run(
      noteId,
      assets,
      recipientDigest,
      metadata,
      nullifier || null,
      expectedHeight,
      stateDiscriminant,
      state
    );
  } catch (error) {
    logWebStoreError(error, `Error inserting note: ${noteId}`);
  }
}

function processInputNotes(db, notes) {
  return notes.map((note) => {
    let serializedNoteScriptBase64 = undefined;
    if (note.scriptRoot) {
      const scriptRecord = db
        .prepare(
          "SELECT serializedNoteScript FROM notes_scripts WHERE scriptRoot = ?"
        )
        .get(note.scriptRoot);
      if (scriptRecord && scriptRecord.serializedNoteScript) {
        serializedNoteScriptBase64 = uint8ArrayToBase64(
          scriptRecord.serializedNoteScript
        );
      }
    }
    return {
      assets: uint8ArrayToBase64(note.assets),
      serialNumber: uint8ArrayToBase64(note.serialNumber),
      inputs: uint8ArrayToBase64(note.inputs),
      createdAt: note.serializedCreatedAt,
      serializedNoteScript: serializedNoteScriptBase64,
      state: uint8ArrayToBase64(note.state),
    };
  });
}

function processOutputNotes(notes) {
  return notes.map((note) => ({
    assets: uint8ArrayToBase64(note.assets),
    recipientDigest: note.recipientDigest,
    metadata: uint8ArrayToBase64(note.metadata),
    expectedHeight: note.expectedHeight,
    state: uint8ArrayToBase64(note.state),
  }));
}

export async function upsertNoteScript(dbId, scriptRoot, serializedNoteScript) {
  const db = getDatabase(dbId);
  try {
    db.prepare(
      "INSERT OR REPLACE INTO notes_scripts (scriptRoot, serializedNoteScript) VALUES (?, ?)"
    ).run(scriptRoot, serializedNoteScript);
  } catch (error) {
    logWebStoreError(error, `Error inserting note script: ${scriptRoot}`);
  }
}
