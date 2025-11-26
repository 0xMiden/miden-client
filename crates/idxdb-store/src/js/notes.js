import { db, inputNotes, outputNotes, notesScripts, } from "./schema.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";
export async function getOutputNotes(states) {
    try {
        let notes = states.length == 0
            ? await outputNotes.toArray()
            : await outputNotes.where("stateDiscriminant").anyOf(states).toArray();
        return await processOutputNotes(notes);
    }
    catch (err) {
        logWebStoreError(err, "Failed to get output notes");
    }
}
export async function getInputNotes(states) {
    try {
        let notes;
        if (states.length === 0) {
            notes = await inputNotes.toArray();
        }
        else {
            notes = await inputNotes
                .where("stateDiscriminant")
                .anyOf(states)
                .toArray();
        }
        return await processInputNotes(notes);
    }
    catch (err) {
        logWebStoreError(err, "Failed to get input notes");
    }
}
export async function getInputNotesFromIds(noteIds) {
    try {
        let notes = await inputNotes.where("noteId").anyOf(noteIds).toArray();
        return await processInputNotes(notes);
    }
    catch (err) {
        logWebStoreError(err, "Failed to get input notes from IDs");
    }
}
export async function getInputNotesFromNullifiers(nullifiers) {
    try {
        let notes = await inputNotes.where("nullifier").anyOf(nullifiers).toArray();
        return await processInputNotes(notes);
    }
    catch (err) {
        logWebStoreError(err, "Failed to get input notes from nullifiers");
    }
}
export async function getOutputNotesFromNullifiers(nullifiers) {
    try {
        let notes = await outputNotes
            .where("nullifier")
            .anyOf(nullifiers)
            .toArray();
        return await processOutputNotes(notes);
    }
    catch (err) {
        logWebStoreError(err, "Failed to get output notes from nullifiers");
    }
}
export async function getOutputNotesFromIds(noteIds) {
    try {
        let notes = await outputNotes.where("noteId").anyOf(noteIds).toArray();
        return await processOutputNotes(notes);
    }
    catch (err) {
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
    }
    catch (err) {
        logWebStoreError(err, "Failed to get unspent input note nullifiers");
    }
}
export async function getNoteScript(scriptRoot) {
    try {
        const noteScript = await notesScripts
            .where("scriptRoot")
            .equals(scriptRoot)
            .first();
        return noteScript;
    }
    catch (err) {
        logWebStoreError(err, "Failed to get note script from root");
    }
}
export async function upsertInputNote(noteId, assets, serialNumber, inputs, scriptRoot, serializedNoteScript, nullifier, serializedCreatedAt, stateDiscriminant, state) {
    return db.transaction("rw", inputNotes, notesScripts, async (tx) => {
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
export async function upsertOutputNote(noteId, assets, recipientDigest, metadata, nullifier, expectedHeight, stateDiscriminant, state) {
    return db.transaction("rw", outputNotes, notesScripts, async (tx) => {
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
async function processInputNotes(notes) {
    return await Promise.all(notes.map(async (note) => {
        const assetsBase64 = uint8ArrayToBase64(note.assets);
        const serialNumberBase64 = uint8ArrayToBase64(note.serialNumber);
        const inputsBase64 = uint8ArrayToBase64(note.inputs);
        let serializedNoteScriptBase64 = undefined;
        if (note.scriptRoot) {
            let record = await notesScripts.get(note.scriptRoot);
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
export async function upsertNoteScript(scriptRoot, serializedNoteScript) {
    return db.transaction("rw", outputNotes, notesScripts, async (tx) => {
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
