export declare function getSettingValue(key: string): Promise<{
    key: string;
    value: string;
} | null | undefined>;
export declare function insertSettingValue(key: string, value: Uint8Array): Promise<void>;
export declare function deleteSettingValue(key: string): Promise<void>;
