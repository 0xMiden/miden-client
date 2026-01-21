import { getDatabase } from "./schema.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";
export async function getOutputNotes(dbId, states) {
    try {
        const db = getDatabase(dbId);
        let notes = states.length == 0
            ? await db.outputNotes.toArray()
            : await db.outputNotes
                .where("stateDiscriminant")
                .anyOf(states)
                .toArray();
        return await processOutputNotes(notes);
    }
    catch (err) {
        logWebStoreError(err, "Failed to get output notes");
    }
}
export async function getInputNotes(dbId, states) {
    try {
        const db = getDatabase(dbId);
        let notes;
        if (states.length === 0) {
            notes = await db.inputNotes.toArray();
        }
        else {
            notes = await db.inputNotes
                .where("stateDiscriminant")
                .anyOf(states)
                .toArray();
        }
        return await processInputNotes(dbId, notes);
    }
    catch (err) {
        logWebStoreError(err, "Failed to get input notes");
    }
}
export async function getInputNotesFromIds(dbId, noteIds) {
    try {
        const db = getDatabase(dbId);
        let notes = await db.inputNotes.where("noteId").anyOf(noteIds).toArray();
        return await processInputNotes(dbId, notes);
    }
    catch (err) {
        logWebStoreError(err, "Failed to get input notes from IDs");
    }
}
export async function getInputNotesFromNullifiers(dbId, nullifiers) {
    try {
        const db = getDatabase(dbId);
        let notes = await db.inputNotes
            .where("nullifier")
            .anyOf(nullifiers)
            .toArray();
        return await processInputNotes(dbId, notes);
    }
    catch (err) {
        logWebStoreError(err, "Failed to get input notes from nullifiers");
    }
}
export async function getOutputNotesFromNullifiers(dbId, nullifiers) {
    try {
        const db = getDatabase(dbId);
        let notes = await db.outputNotes
            .where("nullifier")
            .anyOf(nullifiers)
            .toArray();
        return await processOutputNotes(notes);
    }
    catch (err) {
        logWebStoreError(err, "Failed to get output notes from nullifiers");
    }
}
export async function getOutputNotesFromIds(dbId, noteIds) {
    try {
        const db = getDatabase(dbId);
        let notes = await db.outputNotes.where("noteId").anyOf(noteIds).toArray();
        return await processOutputNotes(notes);
    }
    catch (err) {
        logWebStoreError(err, "Failed to get output notes from IDs");
    }
}
export async function getUnspentInputNoteNullifiers(dbId) {
    try {
        const db = getDatabase(dbId);
        const notes = await db.inputNotes
            .where("stateDiscriminant")
            .anyOf([2, 4, 5])
            .toArray();
        return notes.map((note) => note.nullifier);
    }
    catch (err) {
        logWebStoreError(err, "Failed to get unspent input note nullifiers");
    }
}
export async function getNoteScript(dbId, scriptRoot) {
    try {
        const db = getDatabase(dbId);
        const noteScript = await db.notesScripts
            .where("scriptRoot")
            .equals(scriptRoot)
            .first();
        return noteScript;
    }
    catch (err) {
        logWebStoreError(err, "Failed to get note script from root");
    }
}
export async function upsertInputNote(dbId, noteId, assets, serialNumber, inputs, scriptRoot, serializedNoteScript, nullifier, serializedCreatedAt, stateDiscriminant, state) {
    const db = getDatabase(dbId);
    return db.dexie.transaction("rw", db.inputNotes, db.notesScripts, async (tx) => {
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
            };
            await tx.inputNotes.put(data);
            const noteScriptData = {
                scriptRoot,
                serializedNoteScript,
            };
            await tx.notesScripts.put(noteScriptData);
        }
        catch (error) {
            logWebStoreError(error, `Error inserting note: ${noteId}`);
        }
    });
}
export async function upsertOutputNote(dbId, noteId, assets, recipientDigest, metadata, nullifier, expectedHeight, stateDiscriminant, state) {
    const db = getDatabase(dbId);
    return db.dexie.transaction("rw", db.outputNotes, db.notesScripts, async (tx) => {
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
        }
        catch (error) {
            logWebStoreError(error, `Error inserting note: ${noteId}`);
        }
    });
}
async function processInputNotes(dbId, notes) {
    const db = getDatabase(dbId);
    return await Promise.all(notes.map(async (note) => {
        const assetsBase64 = uint8ArrayToBase64(note.assets);
        const serialNumberBase64 = uint8ArrayToBase64(note.serialNumber);
        const inputsBase64 = uint8ArrayToBase64(note.inputs);
        let serializedNoteScriptBase64 = undefined;
        if (note.scriptRoot) {
            let record = await db.notesScripts.get(note.scriptRoot);
            if (record) {
                serializedNoteScriptBase64 = uint8ArrayToBase64(record.serializedNoteScript);
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
    }));
}
async function processOutputNotes(notes) {
    return await Promise.all(notes.map((note) => {
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
    }));
}
export async function upsertNoteScript(dbId, scriptRoot, serializedNoteScript) {
    const db = getDatabase(dbId);
    return db.dexie.transaction("rw", db.outputNotes, db.notesScripts, async (tx) => {
        try {
            const noteScriptData = {
                scriptRoot,
                serializedNoteScript,
            };
            await tx.notesScripts.put(noteScriptData);
        }
        catch (error) {
            logWebStoreError(error, `Error inserting note script: ${scriptRoot}`);
        }
    });
}
