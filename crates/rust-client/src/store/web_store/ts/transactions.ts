import {
  ITransaction,
  ITransactionScript,
  transactions,
  transactionScripts,
} from "./schema.js";
import { Dexie } from "dexie";
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
    const scriptMap: Map<string, Blob> = new Map();
    scripts.forEach((script) => {
      if (script.txScript) {
        scriptMap.set(script.scriptRoot, script.txScript);
      }
    });

    const processedTransactions = await Promise.all(
      transactionRecords.map(async (transactionRecord) => {
        let txScriptBase64: undefined | string = undefined;
        if (transactionRecord.scriptRoot) {
          const txScript = scriptMap.get(transactionRecord.scriptRoot);

          if (txScript) {
            const txScriptArrayBuffer = await txScript.arrayBuffer();
            const txScriptArray = new Uint8Array(txScriptArrayBuffer);
            txScriptBase64 = uint8ArrayToBase64(txScriptArray);
          }
        }

        const detailsArrayBuffer =
          await transactionRecord.details.arrayBuffer();
        const detailsArray = new Uint8Array(detailsArrayBuffer);
        const detailsBase64 = uint8ArrayToBase64(detailsArray);

        const statusArrayBuffer = await transactionRecord.status.arrayBuffer();
        const statusArray = new Uint8Array(statusArrayBuffer);
        const statusBase64 = uint8ArrayToBase64(statusArray);

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
  scriptRoot?: Uint8Array,
  txScript?: Uint8Array
) {
  if (!scriptRoot) {
    return;
  }

  try {
    // check if script root already exists
    const record = await transactionScripts
      .where("scriptRoot")
      .equals(scriptRoot)
      .first();

    if (record) {
      return;
    }

    const scriptRootArray = new Uint8Array(scriptRoot);
    const scriptRootBase64 = uint8ArrayToBase64(scriptRootArray);

    const data: ITransactionScript = {
      scriptRoot: scriptRootBase64,
      txScript: mapOption(
        txScript,
        (txScript) => new Blob([new Uint8Array(txScript)])
      ),
    };

    await transactionScripts.add(data);
  } catch (error) {
    // Check if the error is because the record already exists
    if (!(error instanceof Dexie.ConstraintError)) {
      logWebStoreError(error, "Failed to insert transaction script");
    }
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
    const detailsBlob = new Blob([new Uint8Array(details)]);
    const statusBlob = new Blob([new Uint8Array(status)]);

    const data = {
      id: transactionId,
      details: detailsBlob,
      scriptRoot: mapOption(scriptRoot, (root) => uint8ArrayToBase64(root)),
      blockNum: parseInt(blockNum, 10),
      statusVariant,
      status: statusBlob,
    };

    await transactions.put(data);
  } catch (err) {
    logWebStoreError(err, "Failed to insert proven transaction data");
  }
}
