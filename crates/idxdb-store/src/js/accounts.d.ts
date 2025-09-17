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
export declare function getAccountStorage(storageRoot: string): Promise<{
    root: string;
    storage: string;
} | null | undefined>;
export declare function getAccountAssetVault(vaultRoot: string): Promise<{
    root: string;
    assets: string;
} | null | undefined>;
export declare function getAccountAuthByPubKey(pubKey: string): Promise<{
    secretKey: string;
}>;
export declare function getAccountAddresses(accountId: string): Promise<import("./schema.js").IAddress[] | undefined>;
export declare function insertAccountCode(codeRoot: string, code: Uint8Array): Promise<void>;
export declare function insertAccountStorage(storageRoot: string, storageSlots: Uint8Array): Promise<void>;
export declare function insertAccountAssetVault(vaultRoot: string, assets: Uint8Array): Promise<void>;
export declare function insertAccountRecord(accountId: string, codeRoot: string, storageRoot: string, vaultRoot: string, nonce: string, committed: boolean, commitment: string, accountSeed: Uint8Array | undefined): Promise<void>;
export declare function insertAccountAuth(pubKey: string, secretKey: string): Promise<void>;
export declare function insertAccountAddress(address: Uint8Array, accountId: string): Promise<void>;
export declare function upsertForeignAccountCode(accountId: string, code: Uint8Array, codeRoot: string): Promise<void>;
export declare function getForeignAccountCode(accountIds: string[]): Promise<{
    accountId: string;
    code: string;
}[] | null | undefined>;
export declare function lockAccount(accountId: string): Promise<void>;
export declare function undoAccountStates(accountCommitments: string[]): Promise<void>;
