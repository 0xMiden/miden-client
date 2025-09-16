interface ProcessedTransaction {
    scriptRoot?: string;
    details?: string;
    id: string;
    txScript?: string;
    blockNum: string;
    statusVariant: number;
    status?: string;
}
export declare function getTransactions(filter: string): Promise<ProcessedTransaction[] | undefined>;
export declare function insertTransactionScript(scriptRoot: Uint8Array, txScript: Uint8Array): Promise<void>;
export declare function upsertTransactionRecord(transactionId: string, details: Uint8Array, blockNum: string, statusVariant: number, status: Uint8Array, scriptRoot?: Uint8Array): Promise<void>;
export {};
