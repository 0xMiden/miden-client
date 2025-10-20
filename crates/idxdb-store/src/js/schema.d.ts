import Dexie from "dexie";
export declare function openDatabase(): Promise<boolean>;
export interface IAccountCode {
    root: string;
    code: Uint8Array;
}
export interface IAccountStorage {
    commitment: string;
    slotIndex: number;
    slotValue: string;
    slotType: number;
}
export interface IStorageMapEntry {
    root: string;
    key: string;
    value: string;
}
export interface IAccountAsset {
    root: string;
    vaultKey: string;
    faucetIdPrefix: string;
    asset: string;
}
export interface IAccountAuth {
    pubKey: string;
    secretKey: string;
}
export interface IAccount {
    id: string;
    codeRoot: string;
    storageRoot: string;
    vaultRoot: string;
    nonce: string;
    committed: boolean;
    accountSeed?: Uint8Array;
    accountCommitment: string;
    locked: boolean;
}
export interface IAddress {
    address: Uint8Array;
    id: string;
}
export interface ITransaction {
    id: string;
    details: Uint8Array;
    blockNum: number;
    scriptRoot?: string;
    statusVariant: number;
    status: Uint8Array;
}
export interface ITransactionScript {
    scriptRoot: string;
    txScript?: Uint8Array;
}
export interface IInputNote {
    noteId: string;
    stateDiscriminant: number;
    assets: Uint8Array;
    serialNumber: Uint8Array;
    inputs: Uint8Array;
    scriptRoot: string;
    nullifier: string;
    serializedCreatedAt: string;
    state: Uint8Array;
}
export interface IOutputNote {
    noteId: string;
    recipientDigest: string;
    assets: Uint8Array;
    metadata: Uint8Array;
    stateDiscriminant: number;
    nullifier?: string;
    expectedHeight: number;
    state: Uint8Array;
}
export interface INotesScript {
    scriptRoot: string;
    serializedNoteScript: Uint8Array;
}
export interface IStateSync {
    id: number;
    blockNum: string;
}
export interface IBlockHeader {
    blockNum: string;
    header: Uint8Array;
    partialBlockchainPeaks: Uint8Array;
    hasClientNotes: string;
}
export interface IPartialBlockchainNode {
    id: string;
    node: string;
}
export interface ITag {
    id?: number;
    tag: string;
    sourceNoteId?: string;
    sourceAccountId?: string;
}
export interface IForeignAccountCode {
    accountId: string;
    codeRoot: string;
}
export interface ISetting {
    key: string;
    value: Uint8Array;
}
declare const db: Dexie & {
    accountCodes: Dexie.Table<IAccountCode, string>;
    accountStorages: Dexie.Table<IAccountStorage, string>;
    accountAssets: Dexie.Table<IAccountAsset, string>;
    storageMapEntries: Dexie.Table<IStorageMapEntry, string>;
    accountAuths: Dexie.Table<IAccountAuth, string>;
    accounts: Dexie.Table<IAccount, string>;
    addresses: Dexie.Table<IAddress, string>;
    transactions: Dexie.Table<ITransaction, string>;
    transactionScripts: Dexie.Table<ITransactionScript, string>;
    inputNotes: Dexie.Table<IInputNote, string>;
    outputNotes: Dexie.Table<IOutputNote, string>;
    notesScripts: Dexie.Table<INotesScript, string>;
    stateSync: Dexie.Table<IStateSync, number>;
    blockHeaders: Dexie.Table<IBlockHeader, string>;
    partialBlockchainNodes: Dexie.Table<IPartialBlockchainNode, string>;
    tags: Dexie.Table<ITag, number>;
    foreignAccountCode: Dexie.Table<IForeignAccountCode, string>;
    settings: Dexie.Table<ISetting, string>;
};
declare const accountCodes: import("dexie").Table<IAccountCode, string, IAccountCode>;
declare const accountStorages: import("dexie").Table<IAccountStorage, string, IAccountStorage>;
declare const storageMapEntries: import("dexie").Table<IStorageMapEntry, string, IStorageMapEntry>;
declare const accountAssets: import("dexie").Table<IAccountAsset, string, IAccountAsset>;
declare const accountAuths: import("dexie").Table<IAccountAuth, string, IAccountAuth>;
declare const accounts: import("dexie").Table<IAccount, string, IAccount>;
declare const addresses: import("dexie").Table<IAddress, string, IAddress>;
declare const transactions: import("dexie").Table<ITransaction, string, ITransaction>;
declare const transactionScripts: import("dexie").Table<ITransactionScript, string, ITransactionScript>;
declare const inputNotes: import("dexie").Table<IInputNote, string, IInputNote>;
declare const outputNotes: import("dexie").Table<IOutputNote, string, IOutputNote>;
declare const notesScripts: import("dexie").Table<INotesScript, string, INotesScript>;
declare const stateSync: import("dexie").Table<IStateSync, number, IStateSync>;
declare const blockHeaders: import("dexie").Table<IBlockHeader, string, IBlockHeader>;
declare const partialBlockchainNodes: import("dexie").Table<IPartialBlockchainNode, string, IPartialBlockchainNode>;
declare const tags: import("dexie").Table<ITag, number, ITag>;
declare const foreignAccountCode: import("dexie").Table<IForeignAccountCode, string, IForeignAccountCode>;
declare const settings: import("dexie").Table<ISetting, string, ISetting>;
export { db, accountCodes, accountStorages, storageMapEntries, accountAssets, accountAuths, accounts, addresses, transactions, transactionScripts, inputNotes, outputNotes, notesScripts, stateSync, blockHeaders, partialBlockchainNodes, tags, foreignAccountCode, settings, };
