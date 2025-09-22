export declare function getValue(key: string): Promise<{
    key: string;
    value: string;
} | null | undefined>;
export declare function insertValue(key: string, value: Uint8Array): Promise<void>;
export declare function removeValue(key: string): Promise<void>;
export declare function listKeys(): Promise<string[] | undefined>;
