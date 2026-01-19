import { getDatabase, ITransaction, ITransactionScript } from "./schema.js";
import { logWebStoreError, mapOption, uint8ArrayToBase64 } from "./utils.js";

interface ProcessedTransaction {
  scriptRoot?: string;
  details?: string;
  id: string;
  txScript?: string;
  blockNum: number;
  statusVariant: number;
  status?: string;
}

const IDS_FILTER_PREFIX = "Ids:";
const EXPIRED_BEFORE_FILTER_PREFIX = "ExpiredPending:";

const STATUS_PENDING_VARIANT = 0;
const STATUS_COMMITTED_VARIANT = 1;
const STATUS_DISCARDED_VARIANT = 2;

export async function getTransactions(dbId: string, filter: string) {
  let transactionRecords: ITransaction[] = [];

  try {
    const db = getDatabase(dbId);
    if (filter === "Uncommitted") {
      transactionRecords = await db.transactions
        .filter((tx) => tx.statusVariant === STATUS_PENDING_VARIANT)
        .toArray();
    } else if (filter.startsWith(IDS_FILTER_PREFIX)) {
      const idsString = filter.substring(IDS_FILTER_PREFIX.length);
      const ids = idsString.split(",");

      if (ids.length > 0) {
        transactionRecords = await db.transactions
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

      transactionRecords = await db.transactions
        .filter(
          (tx) =>
            tx.blockNum < blockNum &&
            tx.statusVariant !== STATUS_COMMITTED_VARIANT &&
            tx.statusVariant !== STATUS_DISCARDED_VARIANT
        )
        .toArray();
    } else {
      transactionRecords = await db.transactions.toArray();
    }

    if (transactionRecords.length === 0) {
      return [];
    }

    const scriptRoots = transactionRecords
      .map((transactionRecord) => {
        return transactionRecord.scriptRoot;
      })
      .filter((scriptRoot) => scriptRoot != undefined);

    const scripts = await db.transactionScripts
      .where("scriptRoot")
      .anyOf(scriptRoots)
      .toArray();

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
          blockNum: transactionRecord.blockNum,
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
  dbId: string,
  scriptRoot: Uint8Array,
  txScript: Uint8Array
) {
  try {
    const db = getDatabase(dbId);
    const scriptRootArray = new Uint8Array(scriptRoot);
    const scriptRootBase64 = uint8ArrayToBase64(scriptRootArray);

    const data: ITransactionScript = {
      scriptRoot: scriptRootBase64,
      txScript: mapOption(txScript, (txScript) => new Uint8Array(txScript)),
    };

    await db.transactionScripts.put(data);
  } catch (error) {
    logWebStoreError(error, "Failed to insert transaction script");
  }
}

export async function upsertTransactionRecord(
  dbId: string,
  transactionId: string,
  details: Uint8Array,
  blockNum: number,
  statusVariant: number,
  status: Uint8Array,
  scriptRoot?: Uint8Array
) {
  try {
    const db = getDatabase(dbId);
    const data = {
      id: transactionId,
      details,
      scriptRoot: mapOption(scriptRoot, (root) => uint8ArrayToBase64(root)),
      blockNum,
      statusVariant,
      status,
    };

    await db.transactions.put(data);
  } catch (err) {
    logWebStoreError(err, "Failed to insert proven transaction data");
  }
}
