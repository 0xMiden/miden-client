export declare function getSetting(key: string): Promise<{
    key: string;
    value: string;
} | null | undefined>;
export declare function insertSetting(key: string, value: Uint8Array): Promise<void>;
export declare function removeSetting(key: string): Promise<void>;
export declare function listSettingKeys(): Promise<string[] | undefined>;
