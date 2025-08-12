import {
  ITransaction,
  ITransactionScript,
  transactions,
  transactionScripts,
} from "./schema.js";
import { Dexie } from "dexie";
import { logDexieError, mapOption, uint8ArrayToBase64 } from "./utils.js";

interface ProcessedTransaction {
  scriptRoot?: string;
  discardCause?: string;
  details?: string;
  id: string;
  txScript?: string;
  blockNum: string;
  commitHeight?: string;
}

const IDS_FILTER_PREFIX = "Ids:";
export async function getTransactions(filter: string) {
  let transactionRecords: ITransaction[] = [];

  try {
    if (filter === "Uncommitted") {
      transactionRecords = await transactions
        .filter((tx) => tx.commitHeight === undefined)
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
        let discardCauseBase64: string | undefined = undefined;
        if (transactionRecord.scriptRoot) {
          const txScript = scriptMap.get(transactionRecord.scriptRoot);

          if (txScript) {
            const txScriptArrayBuffer = await txScript.arrayBuffer();
            const txScriptArray = new Uint8Array(txScriptArrayBuffer);
            txScriptBase64 = uint8ArrayToBase64(txScriptArray);
          }
        }

        if (transactionRecord.discardCause) {
          const discardCauseArrayBuffer =
            await transactionRecord.discardCause.arrayBuffer();
          const discardCauseArray = new Uint8Array(discardCauseArrayBuffer);
          discardCauseBase64 = uint8ArrayToBase64(discardCauseArray);
        }

        const detailsArrayBuffer =
          await transactionRecord.details.arrayBuffer();
        const detailsArray = new Uint8Array(detailsArrayBuffer);
        const detailsBase64 = uint8ArrayToBase64(detailsArray);

        const data: ProcessedTransaction = {
          id: transactionRecord.id,
          details: detailsBase64,
          scriptRoot: transactionRecord.scriptRoot,
          txScript: txScriptBase64,
          blockNum: transactionRecord.blockNum,
          commitHeight: transactionRecord.commitHeight,
          discardCause: discardCauseBase64,
        };

        return data;
      })
    );

    return processedTransactions;
  } catch (err) {
    logDexieError(err, "Failed to get transactions");
  }
}

export async function insertTransactionScript(
  scriptRoot: Uint8Array,
  txScript?: Uint8Array
) {
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
      logDexieError(error, "Failed to insert transaction script");
    }
  }
}

// FIXME: Add to changelog that these parameters
// have change order.
export async function upsertTransactionRecord(
  transactionId: string,
  details: Uint8Array,
  blockNum: string,
  scriptRoot?: Uint8Array,
  committed?: string,
  discardCause?: Uint8Array
) {
  try {
    const detailsBlob = new Blob([new Uint8Array(details)]);

    const data = {
      id: transactionId,
      details: detailsBlob,
      scriptRoot: mapOption(scriptRoot, (root) => uint8ArrayToBase64(root)),
      blockNum: blockNum,
      commitHeight: committed ? committed : undefined,
      discardCause: mapOption(
        discardCause,
        (discardCause) => new Blob([discardCause as BlobPart])
      ),
    };

    await transactions.put(data);
  } catch (err) {
    logDexieError(err, "Failed to insert proven transaction data");
  }
}
