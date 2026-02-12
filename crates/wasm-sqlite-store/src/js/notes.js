/**
 * Note operations for the WASM SQLite store.
 */
import { getDatabase } from "./schema.js";
import { logError, uint8ArrayToBase64 } from "./utils.js";
function toBase64(data) {
  if (!data) return "";
  const bytes = data instanceof Uint8Array ? data : new Uint8Array(data);
  return uint8ArrayToBase64(bytes);
}
export function getInputNotes(dbId, states) {
  try {
    const db = getDatabase(dbId);
    let rows;
    if (states.length === 0) {
      rows =
        db.all(`SELECT note.note_id, note.assets, note.serial_number, note.inputs,
                note.script_root, script.serialized_note_script, note.nullifier,
                note.state_discriminant, note.state, note.created_at
         FROM input_notes AS note
         LEFT OUTER JOIN notes_scripts AS script ON note.script_root = script.script_root`);
    } else {
      const placeholders = states.map(() => "?").join(",");
      rows = db.all(
        `SELECT note.note_id, note.assets, note.serial_number, note.inputs,
                note.script_root, script.serialized_note_script, note.nullifier,
                note.state_discriminant, note.state, note.created_at
         FROM input_notes AS note
         LEFT OUTER JOIN notes_scripts AS script ON note.script_root = script.script_root
         WHERE note.state_discriminant IN (${placeholders})`,
        states
      );
    }
    return rows.map((row) => ({
      noteId: row.note_id,
      assets: toBase64(row.assets),
      serialNumber: toBase64(row.serial_number),
      inputs: toBase64(row.inputs),
      scriptRoot: row.script_root,
      serializedNoteScript: toBase64(row.serialized_note_script),
      nullifier: row.nullifier,
      stateDiscriminant: row.state_discriminant,
      state: toBase64(row.state),
      createdAt: row.created_at.toString(),
    }));
  } catch (error) {
    logError(error, "Error fetching input notes");
    return [];
  }
}
export function getInputNotesFromIds(dbId, noteIds) {
  try {
    if (noteIds.length === 0) return [];
    const db = getDatabase(dbId);
    const placeholders = noteIds.map(() => "?").join(",");
    const rows = db.all(
      `SELECT note.note_id, note.assets, note.serial_number, note.inputs,
              note.script_root, script.serialized_note_script, note.nullifier,
              note.state_discriminant, note.state, note.created_at
       FROM input_notes AS note
       LEFT OUTER JOIN notes_scripts AS script ON note.script_root = script.script_root
       WHERE note.note_id IN (${placeholders})`,
      noteIds
    );
    return rows.map((row) => ({
      noteId: row.note_id,
      assets: toBase64(row.assets),
      serialNumber: toBase64(row.serial_number),
      inputs: toBase64(row.inputs),
      scriptRoot: row.script_root,
      serializedNoteScript: toBase64(row.serialized_note_script),
      nullifier: row.nullifier,
      stateDiscriminant: row.state_discriminant,
      state: toBase64(row.state),
      createdAt: row.created_at.toString(),
    }));
  } catch (error) {
    logError(error, "Error fetching input notes by ids");
    return [];
  }
}
export function getInputNotesFromNullifiers(dbId, nullifiers) {
  try {
    if (nullifiers.length === 0) return [];
    const db = getDatabase(dbId);
    const placeholders = nullifiers.map(() => "?").join(",");
    const rows = db.all(
      `SELECT note.note_id, note.assets, note.serial_number, note.inputs,
              note.script_root, script.serialized_note_script, note.nullifier,
              note.state_discriminant, note.state, note.created_at
       FROM input_notes AS note
       LEFT OUTER JOIN notes_scripts AS script ON note.script_root = script.script_root
       WHERE note.nullifier IN (${placeholders})`,
      nullifiers
    );
    return rows.map((row) => ({
      noteId: row.note_id,
      assets: toBase64(row.assets),
      serialNumber: toBase64(row.serial_number),
      inputs: toBase64(row.inputs),
      scriptRoot: row.script_root,
      serializedNoteScript: toBase64(row.serialized_note_script),
      nullifier: row.nullifier,
      stateDiscriminant: row.state_discriminant,
      state: toBase64(row.state),
      createdAt: row.created_at.toString(),
    }));
  } catch (error) {
    logError(error, "Error fetching input notes by nullifiers");
    return [];
  }
}
export function getOutputNotes(dbId, states) {
  try {
    const db = getDatabase(dbId);
    let rows;
    if (states.length === 0) {
      rows =
        db.all(`SELECT note_id, recipient_digest, assets, metadata, nullifier,
                expected_height, state_discriminant, state
         FROM output_notes`);
    } else {
      const placeholders = states.map(() => "?").join(",");
      rows = db.all(
        `SELECT note_id, recipient_digest, assets, metadata, nullifier,
                expected_height, state_discriminant, state
         FROM output_notes
         WHERE state_discriminant IN (${placeholders})`,
        states
      );
    }
    return rows.map((row) => ({
      noteId: row.note_id,
      recipientDigest: row.recipient_digest,
      assets: toBase64(row.assets),
      metadata: toBase64(row.metadata),
      nullifier: row.nullifier,
      expectedHeight: row.expected_height,
      stateDiscriminant: row.state_discriminant,
      state: toBase64(row.state),
    }));
  } catch (error) {
    logError(error, "Error fetching output notes");
    return [];
  }
}
export function getOutputNotesFromIds(dbId, noteIds) {
  try {
    if (noteIds.length === 0) return [];
    const db = getDatabase(dbId);
    const placeholders = noteIds.map(() => "?").join(",");
    const rows = db.all(
      `SELECT note_id, recipient_digest, assets, metadata, nullifier,
              expected_height, state_discriminant, state
       FROM output_notes
       WHERE note_id IN (${placeholders})`,
      noteIds
    );
    return rows.map((row) => ({
      noteId: row.note_id,
      recipientDigest: row.recipient_digest,
      assets: toBase64(row.assets),
      metadata: toBase64(row.metadata),
      nullifier: row.nullifier,
      expectedHeight: row.expected_height,
      stateDiscriminant: row.state_discriminant,
      state: toBase64(row.state),
    }));
  } catch (error) {
    logError(error, "Error fetching output notes by ids");
    return [];
  }
}
export function getOutputNotesFromNullifiers(dbId, nullifiers) {
  try {
    if (nullifiers.length === 0) return [];
    const db = getDatabase(dbId);
    const placeholders = nullifiers.map(() => "?").join(",");
    const rows = db.all(
      `SELECT note_id, recipient_digest, assets, metadata, nullifier,
              expected_height, state_discriminant, state
       FROM output_notes
       WHERE nullifier IN (${placeholders})`,
      nullifiers
    );
    return rows.map((row) => ({
      noteId: row.note_id,
      recipientDigest: row.recipient_digest,
      assets: toBase64(row.assets),
      metadata: toBase64(row.metadata),
      nullifier: row.nullifier,
      expectedHeight: row.expected_height,
      stateDiscriminant: row.state_discriminant,
      state: toBase64(row.state),
    }));
  } catch (error) {
    logError(error, "Error fetching output notes by nullifiers");
    return [];
  }
}
export function getUnspentInputNoteNullifiers(dbId) {
  try {
    const db = getDatabase(dbId);
    // State discriminants for unspent notes: Expected(2), Committed(4), Processing(5), Unverified(6)
    const rows = db.all(
      `SELECT nullifier FROM input_notes WHERE state_discriminant IN (2, 4, 5, 6)`
    );
    return rows.map((row) => row.nullifier);
  } catch (error) {
    logError(error, "Error fetching unspent input note nullifiers");
    return [];
  }
}
export function getNoteScript(dbId, scriptRoot) {
  try {
    const db = getDatabase(dbId);
    const row = db.get(
      "SELECT script_root, serialized_note_script FROM notes_scripts WHERE script_root = ?",
      [scriptRoot]
    );
    if (!row) return null;
    return {
      scriptRoot: row.script_root,
      serializedNoteScript: toBase64(row.serialized_note_script),
    };
  } catch (error) {
    logError(error, `Error fetching note script: ${scriptRoot}`);
    return null;
  }
}
export function upsertInputNote(
  dbId,
  noteId,
  assets,
  serialNumber,
  inputs,
  noteScriptRoot,
  serializedNoteScript,
  nullifier,
  serializedCreatedAt,
  stateDiscriminant,
  state
) {
  try {
    const db = getDatabase(dbId);
    db.transaction(() => {
      // Upsert note script
      db.run(
        "INSERT OR REPLACE INTO notes_scripts (script_root, serialized_note_script) VALUES (?, ?)",
        [noteScriptRoot, serializedNoteScript]
      );
      // Upsert input note
      db.run(
        `INSERT OR REPLACE INTO input_notes
         (note_id, assets, serial_number, inputs, script_root, nullifier,
          state_discriminant, state, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)`,
        [
          noteId,
          assets,
          serialNumber,
          inputs,
          noteScriptRoot,
          nullifier,
          stateDiscriminant,
          state,
          parseInt(serializedCreatedAt),
        ]
      );
    });
  } catch (error) {
    logError(error, `Error upserting input note: ${noteId}`);
  }
}
export function upsertOutputNote(
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
  try {
    const db = getDatabase(dbId);
    db.run(
      `INSERT OR REPLACE INTO output_notes
       (note_id, recipient_digest, assets, metadata, nullifier,
        expected_height, state_discriminant, state)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?)`,
      [
        noteId,
        recipientDigest,
        assets,
        metadata,
        nullifier,
        expectedHeight,
        stateDiscriminant,
        state,
      ]
    );
  } catch (error) {
    logError(error, `Error upserting output note: ${noteId}`);
  }
}
export function upsertNoteScript(dbId, noteScriptRoot, serializedNoteScript) {
  try {
    const db = getDatabase(dbId);
    db.run(
      "INSERT OR REPLACE INTO notes_scripts (script_root, serialized_note_script) VALUES (?, ?)",
      [noteScriptRoot, serializedNoteScript]
    );
  } catch (error) {
    logError(error, `Error upserting note script: ${noteScriptRoot}`);
  }
}
