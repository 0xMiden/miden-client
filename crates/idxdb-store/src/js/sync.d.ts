export declare function getNoteTags(): Promise<import("./schema.js").ITag[] | undefined>;
export declare function getSyncHeight(): Promise<{
    blockNum: string;
} | null | undefined>;
export declare function addNoteTag(tag: Uint8Array, sourceNoteId: string, sourceAccountId: string): Promise<void>;
export declare function removeNoteTag(tag: Uint8Array, sourceNoteId?: string, sourceAccountId?: string): Promise<number | undefined>;
interface FlattenedU8Vec {
    data(): Uint8Array;
    lengths(): number[];
}
interface SerializedInputNoteData {
    noteId: string;
    noteAssets: Uint8Array;
    serialNumber: Uint8Array;
    inputs: Uint8Array;
    noteScriptRoot: string;
    noteScript: Uint8Array;
    nullifier: string;
    createdAt: string;
    stateDiscriminant: number;
    state: Uint8Array;
}
interface SerializedOutputNoteData {
    noteId: string;
    noteAssets: Uint8Array;
    recipientDigest: string;
    metadata: Uint8Array;
    nullifier?: string;
    expectedHeight: number;
    stateDiscriminant: number;
    state: Uint8Array;
}
interface SerializedTransactionData {
    id: string;
    details: Uint8Array;
    blockNum: string;
    scriptRoot?: Uint8Array;
    statusVariant: number;
    status: Uint8Array;
    txScript?: Uint8Array;
}
interface JsAccountUpdate {
    storageRoot: string;
    storageSlots: JsStorageSlot[];
    storageMapEntries: JsStorageMapEntry[];
    assetVaultRoot: string;
    assets: JsVaultAsset[];
    accountId: string;
    codeRoot: string;
    committed: boolean;
    nonce: string;
    accountCommitment: string;
    accountSeed?: Uint8Array;
}
interface JsStateSyncUpdate {
    blockNum: string;
    flattenedNewBlockHeaders: FlattenedU8Vec;
    flattenedPartialBlockChainPeaks: FlattenedU8Vec;
    newBlockNums: string[];
    blockHasRelevantNotes: Uint8Array;
    serializedNodeIds: string[];
    serializedNodes: string[];
    committedNoteIds: string[];
    serializedInputNotes: SerializedInputNoteData[];
    serializedOutputNotes: SerializedOutputNoteData[];
    accountUpdates: JsAccountUpdate[];
    transactionUpdates: SerializedTransactionData[];
}
export interface JsVaultAsset {
    root: string;
    vaultKey: string;
    faucetIdPrefix: string;
    asset: string;
}
export interface JsStorageSlot {
    commitment: string;
    slotIndex: number;
    slotValue: string;
    slotType: number;
}
export interface JsStorageMapEntry {
    root: string;
    key: string;
    value: string;
}
export declare function applyStateSync(stateUpdate: JsStateSyncUpdate): Promise<void>;
export {};
