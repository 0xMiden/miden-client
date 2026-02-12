/**
 * Note operations for the WASM SQLite store.
 */

import { getDatabase } from "./schema.js";
import { logError, uint8ArrayToBase64 } from "./utils.js";

export interface InputNoteRow {
  noteId: string;
  assets: string; // base64
  serialNumber: string; // base64
  inputs: string; // base64
  scriptRoot: string;
  serializedNoteScript: string; // base64
  nullifier: string;
  stateDiscriminant: number;
  state: string; // base64
  createdAt: string;
}

export interface OutputNoteRow {
  noteId: string;
  recipientDigest: string;
  assets: string; // base64
  metadata: string; // base64
  nullifier: string | null;
  expectedHeight: number;
  stateDiscriminant: number;
  state: string; // base64
}

export interface NoteScriptRow {
  scriptRoot: string;
  serializedNoteScript: string; // base64
}

function toBase64(data: Uint8Array | ArrayBuffer | null): string {
  if (!data) return "";
  const bytes = data instanceof Uint8Array ? data : new Uint8Array(data);
  return uint8ArrayToBase64(bytes);
}

export function getInputNotes(dbId: string, states: number[]): InputNoteRow[] {
  try {
    const db = getDatabase(dbId);
    let rows;
    if (states.length === 0) {
      rows = db.all<{
        note_id: string;
        assets: Uint8Array;
        serial_number: Uint8Array;
        inputs: Uint8Array;
        script_root: string;
        serialized_note_script: Uint8Array;
        nullifier: string;
        state_discriminant: number;
        state: Uint8Array;
        created_at: number;
      }>(
        `SELECT note.note_id, note.assets, note.serial_number, note.inputs,
                note.script_root, script.serialized_note_script, note.nullifier,
                note.state_discriminant, note.state, note.created_at
         FROM input_notes AS note
         LEFT OUTER JOIN notes_scripts AS script ON note.script_root = script.script_root`
      );
    } else {
      const placeholders = states.map(() => "?").join(",");
      rows = db.all<{
        note_id: string;
        assets: Uint8Array;
        serial_number: Uint8Array;
        inputs: Uint8Array;
        script_root: string;
        serialized_note_script: Uint8Array;
        nullifier: string;
        state_discriminant: number;
        state: Uint8Array;
        created_at: number;
      }>(
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

export function getInputNotesFromIds(
  dbId: string,
  noteIds: string[]
): InputNoteRow[] {
  try {
    if (noteIds.length === 0) return [];
    const db = getDatabase(dbId);
    const placeholders = noteIds.map(() => "?").join(",");
    const rows = db.all<{
      note_id: string;
      assets: Uint8Array;
      serial_number: Uint8Array;
      inputs: Uint8Array;
      script_root: string;
      serialized_note_script: Uint8Array;
      nullifier: string;
      state_discriminant: number;
      state: Uint8Array;
      created_at: number;
    }>(
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

export function getInputNotesFromNullifiers(
  dbId: string,
  nullifiers: string[]
): InputNoteRow[] {
  try {
    if (nullifiers.length === 0) return [];
    const db = getDatabase(dbId);
    const placeholders = nullifiers.map(() => "?").join(",");
    const rows = db.all<{
      note_id: string;
      assets: Uint8Array;
      serial_number: Uint8Array;
      inputs: Uint8Array;
      script_root: string;
      serialized_note_script: Uint8Array;
      nullifier: string;
      state_discriminant: number;
      state: Uint8Array;
      created_at: number;
    }>(
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

export function getOutputNotes(
  dbId: string,
  states: number[]
): OutputNoteRow[] {
  try {
    const db = getDatabase(dbId);
    let rows;
    if (states.length === 0) {
      rows = db.all<{
        note_id: string;
        recipient_digest: string;
        assets: Uint8Array;
        metadata: Uint8Array;
        nullifier: string | null;
        expected_height: number;
        state_discriminant: number;
        state: Uint8Array;
      }>(
        `SELECT note_id, recipient_digest, assets, metadata, nullifier,
                expected_height, state_discriminant, state
         FROM output_notes`
      );
    } else {
      const placeholders = states.map(() => "?").join(",");
      rows = db.all<{
        note_id: string;
        recipient_digest: string;
        assets: Uint8Array;
        metadata: Uint8Array;
        nullifier: string | null;
        expected_height: number;
        state_discriminant: number;
        state: Uint8Array;
      }>(
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

export function getOutputNotesFromIds(
  dbId: string,
  noteIds: string[]
): OutputNoteRow[] {
  try {
    if (noteIds.length === 0) return [];
    const db = getDatabase(dbId);
    const placeholders = noteIds.map(() => "?").join(",");
    const rows = db.all<{
      note_id: string;
      recipient_digest: string;
      assets: Uint8Array;
      metadata: Uint8Array;
      nullifier: string | null;
      expected_height: number;
      state_discriminant: number;
      state: Uint8Array;
    }>(
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

export function getOutputNotesFromNullifiers(
  dbId: string,
  nullifiers: string[]
): OutputNoteRow[] {
  try {
    if (nullifiers.length === 0) return [];
    const db = getDatabase(dbId);
    const placeholders = nullifiers.map(() => "?").join(",");
    const rows = db.all<{
      note_id: string;
      recipient_digest: string;
      assets: Uint8Array;
      metadata: Uint8Array;
      nullifier: string | null;
      expected_height: number;
      state_discriminant: number;
      state: Uint8Array;
    }>(
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

export function getUnspentInputNoteNullifiers(dbId: string): string[] {
  try {
    const db = getDatabase(dbId);
    // State discriminants for unspent notes: Expected(2), Committed(4), Processing(5), Unverified(6)
    const rows = db.all<{ nullifier: string }>(
      `SELECT nullifier FROM input_notes WHERE state_discriminant IN (2, 4, 5, 6)`
    );
    return rows.map((row) => row.nullifier);
  } catch (error) {
    logError(error, "Error fetching unspent input note nullifiers");
    return [];
  }
}

export function getNoteScript(
  dbId: string,
  scriptRoot: string
): NoteScriptRow | null {
  try {
    const db = getDatabase(dbId);
    const row = db.get<{
      script_root: string;
      serialized_note_script: Uint8Array;
    }>(
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
  dbId: string,
  noteId: string,
  assets: Uint8Array,
  serialNumber: Uint8Array,
  inputs: Uint8Array,
  noteScriptRoot: string,
  serializedNoteScript: Uint8Array,
  nullifier: string,
  serializedCreatedAt: string,
  stateDiscriminant: number,
  state: Uint8Array
): void {
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
  dbId: string,
  noteId: string,
  assets: Uint8Array,
  recipientDigest: string,
  metadata: Uint8Array,
  nullifier: string | null,
  expectedHeight: number,
  stateDiscriminant: number,
  state: Uint8Array
): void {
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

export function upsertNoteScript(
  dbId: string,
  noteScriptRoot: string,
  serializedNoteScript: Uint8Array
): void {
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
