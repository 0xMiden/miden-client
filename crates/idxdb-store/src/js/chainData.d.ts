export declare function insertBlockHeader(blockNum: string, header: Uint8Array, partialBlockchainPeaks: Uint8Array, hasClientNotes: boolean): Promise<void>;
export declare function insertPartialBlockchainNodes(ids: string[], nodes: string[]): Promise<void>;
export declare function getBlockHeaders(blockNumbers: string[]): Promise<({
    blockNum: string;
    header: string;
    partialBlockchainPeaks: string;
    hasClientNotes: boolean;
} | null)[] | undefined>;
export declare function getTrackedBlockHeaders(): Promise<{
    blockNum: string;
    header: string;
    partialBlockchainPeaks: string;
    hasClientNotes: boolean;
}[] | undefined>;
export declare function getPartialBlockchainPeaksByBlockNum(blockNum: string): Promise<{
    peaks: undefined;
} | {
    peaks: string;
} | undefined>;
export declare function getPartialBlockchainNodesAll(): Promise<import("./schema.js").IPartialBlockchainNode[] | undefined>;
export declare function getPartialBlockchainNodes(ids: string[]): Promise<(import("./schema.js").IPartialBlockchainNode | undefined)[] | undefined>;
export declare function pruneIrrelevantBlocks(): Promise<void>;
