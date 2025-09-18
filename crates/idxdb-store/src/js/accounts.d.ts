import { IStorageMapEntry } from "./schema.js";
import { JsStorageMapEntry, JsStorageSlot, JsVaultAsset } from "./sync.js";
export declare function getAccountIds(): Promise<unknown[] | undefined>;
export declare function getAllAccountHeaders(): Promise<{
    id: string;
    nonce: string;
    vaultRoot: string;
    storageRoot: string;
    codeRoot: string;
    accountSeed: string | undefined;
    locked: boolean;
    committed: boolean;
    accountCommitment: string;
}[] | undefined>;
export declare function getAccountHeader(accountId: string): Promise<{
    id: string;
    nonce: string;
    vaultRoot: string;
    storageRoot: string;
    codeRoot: string;
    accountSeed: string | undefined;
    locked: boolean;
} | null | undefined>;
export declare function getAccountHeaderByCommitment(accountCommitment: string): Promise<{
    id: string;
    nonce: string;
    vaultRoot: string;
    storageRoot: string;
    codeRoot: string;
    accountSeed: string | undefined;
    locked: boolean;
} | null | undefined>;
export declare function getAccountCode(codeRoot: string): Promise<{
    root: string;
    code: string;
} | null | undefined>;
export declare function getAccountStorage(storageCommitment: string): Promise<{
    slotIndex: number;
    slotValue: string;
    slotType: number;
}[] | undefined>;
export declare function getAccountStorageMaps(roots: string[]): Promise<IStorageMapEntry[] | undefined>;
export declare function getAccountVaultAssets(vaultRoot: string): Promise<{
    asset: string;
}[] | undefined>;
export declare function getAccountAuthByPubKey(pubKey: string): Promise<{
    secretKey: string;
}>;
export declare function insertAccountCode(codeRoot: string, code: Uint8Array): Promise<void>;
export declare function insertAccountStorage(storageSlots: JsStorageSlot[]): Promise<void>;
export declare function insertStorageMapEntries(entries: JsStorageMapEntry[]): Promise<void>;
export declare function insertVaultAssets(assets: JsVaultAsset[]): Promise<void>;
export declare function insertAccountRecord(accountId: string, codeRoot: string, storageRoot: string, vaultRoot: string, nonce: string, committed: boolean, commitment: string, accountSeed: Uint8Array | undefined): Promise<void>;
export declare function insertAccountAuth(pubKey: string, secretKey: string): Promise<void>;
export declare function upsertForeignAccountCode(accountId: string, code: Uint8Array, codeRoot: string): Promise<void>;
export declare function getForeignAccountCode(accountIds: string[]): Promise<{
    accountId: string;
    code: string;
}[] | null | undefined>;
export declare function lockAccount(accountId: string): Promise<void>;
export declare function undoAccountStates(accountCommitments: string[]): Promise<void>;
