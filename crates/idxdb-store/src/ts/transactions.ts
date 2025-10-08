import {
  ITransaction,
  ITransactionScript,
  transactions,
  transactionScripts,
} from "./schema.js";
import { logWebStoreError, mapOption, uint8ArrayToBase64 } from "./utils.js";

interface ProcessedTransaction {
  scriptRoot?: string;
  details?: string;
  id: string;
  txScript?: string;
  blockNum: string;
  statusVariant: number;
  status?: string;
}

const IDS_FILTER_PREFIX = "Ids:";
const EXPIRED_BEFORE_FILTER_PREFIX = "ExpiredPending:";

const STATUS_COMMITTED_VARIANT = 1;
const STATUS_DISCARDED_VARIANT = 2;

export async function getTransactions(filter: string) {
  let transactionRecords: ITransaction[] = [];

  try {
    if (filter === "Uncommitted") {
      transactionRecords = await transactions
        .filter((tx) => tx.statusVariant !== STATUS_COMMITTED_VARIANT)
        .toArray();
    } else if (filter.startsWith(IDS_FILTER_PREFIX)) {
      const idsString = filter.substring(IDS_FILTER_PREFIX.length);
      const ids = idsString.split(",");

      if (ids.length > 0) {
        transactionRecords = await transactions
          .where("id")
          .anyOf(ids)
          .toArray();
      } else {
        transactionRecords = [];
      }
    } else if (filter.startsWith(EXPIRED_BEFORE_FILTER_PREFIX)) {
      const blockNumString = filter.substring(
        EXPIRED_BEFORE_FILTER_PREFIX.length
      );
      const blockNum = parseInt(blockNumString);

      transactionRecords = await transactions
        .filter(
          (tx) =>
            tx.blockNum < blockNum &&
            tx.statusVariant !== STATUS_COMMITTED_VARIANT &&
            tx.statusVariant !== STATUS_DISCARDED_VARIANT
        )
        .toArray();
    } else {
      transactionRecords = await transactions.toArray();
    }

    if (transactionRecords.length === 0) {
      return [];
    }

    const scriptRoots = transactionRecords
      .map((transactionRecord) => {
        return transactionRecord.scriptRoot;
      })
      .filter((scriptRoot) => scriptRoot != undefined);

    const scripts = await transactionScripts
      .where("scriptRoot")
      .anyOf(scriptRoots)
      .toArray();

    // Create a map of scriptRoot to script for quick lookup
    const scriptMap: Map<string, Uint8Array> = new Map();
    scripts.forEach((script) => {
      if (script.txScript) {
        scriptMap.set(script.scriptRoot, script.txScript);
      }
    });

    const processedTransactions = await Promise.all(
      transactionRecords.map((transactionRecord) => {
        let txScriptBase64: undefined | string = undefined;
        if (transactionRecord.scriptRoot) {
          const txScript = scriptMap.get(transactionRecord.scriptRoot);

          if (txScript) {
            txScriptBase64 = uint8ArrayToBase64(txScript);
          }
        }

        const detailsBase64 = uint8ArrayToBase64(transactionRecord.details);

        const statusBase64 = uint8ArrayToBase64(transactionRecord.status);

        const data: ProcessedTransaction = {
          id: transactionRecord.id,
          details: detailsBase64,
          scriptRoot: transactionRecord.scriptRoot,
          txScript: txScriptBase64,
          blockNum: transactionRecord.blockNum.toString(),
          statusVariant: transactionRecord.statusVariant,
          status: statusBase64,
        };

        return data;
      })
    );

    return processedTransactions;
  } catch (err) {
    logWebStoreError(err, "Failed to get transactions");
  }
}

export async function insertTransactionScript(
  scriptRoot: Uint8Array,
  txScript: Uint8Array
) {
  try {
    const scriptRootArray = new Uint8Array(scriptRoot);
    const scriptRootBase64 = uint8ArrayToBase64(scriptRootArray);

    const data: ITransactionScript = {
      scriptRoot: scriptRootBase64,
      txScript: mapOption(txScript, (txScript) => new Uint8Array(txScript)),
    };

    await transactionScripts.put(data);
  } catch (error) {
    logWebStoreError(error, "Failed to insert transaction script");
  }
}

export async function upsertTransactionRecord(
  transactionId: string,
  details: Uint8Array,
  blockNum: string,
  statusVariant: number,
  status: Uint8Array,
  scriptRoot?: Uint8Array
) {
  try {
    const data = {
      id: transactionId,
      details,
      scriptRoot: mapOption(scriptRoot, (root) => uint8ArrayToBase64(root)),
      blockNum: parseInt(blockNum, 10),
      statusVariant,
      status,
    };

    await transactions.put(data);
  } catch (err) {
    logWebStoreError(err, "Failed to insert proven transaction data");
  }
}
