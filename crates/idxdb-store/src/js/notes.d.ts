export declare function getOutputNotes(states: Uint8Array): Promise<{
    assets: string;
    recipientDigest: string;
    metadata: string;
    expectedHeight: number;
    state: string;
}[] | undefined>;
export declare function getInputNotes(states: Uint8Array): Promise<{
    assets: string;
    serialNumber: string;
    inputs: string;
    createdAt: string;
    serializedNoteScript: string | undefined;
    state: string;
}[] | undefined>;
export declare function getInputNotesFromIds(noteIds: string[]): Promise<{
    assets: string;
    serialNumber: string;
    inputs: string;
    createdAt: string;
    serializedNoteScript: string | undefined;
    state: string;
}[] | undefined>;
export declare function getInputNotesFromNullifiers(nullifiers: string[]): Promise<{
    assets: string;
    serialNumber: string;
    inputs: string;
    createdAt: string;
    serializedNoteScript: string | undefined;
    state: string;
}[] | undefined>;
export declare function getOutputNotesFromNullifiers(nullifiers: string[]): Promise<{
    assets: string;
    recipientDigest: string;
    metadata: string;
    expectedHeight: number;
    state: string;
}[] | undefined>;
export declare function getOutputNotesFromIds(noteIds: string[]): Promise<{
    assets: string;
    recipientDigest: string;
    metadata: string;
    expectedHeight: number;
    state: string;
}[] | undefined>;
export declare function getUnspentInputNoteNullifiers(): Promise<string[] | undefined>;
export declare function upsertInputNote(noteId: string, assets: Uint8Array, serialNumber: Uint8Array, inputs: Uint8Array, scriptRoot: string, serializedNoteScript: Uint8Array, nullifier: string, serializedCreatedAt: string, stateDiscriminant: number, state: Uint8Array): Promise<void>;
export declare function upsertOutputNote(noteId: string, assets: Uint8Array, recipientDigest: string, metadata: Uint8Array, nullifier: string | undefined, expectedHeight: number, stateDiscriminant: number, state: Uint8Array): Promise<void>;
