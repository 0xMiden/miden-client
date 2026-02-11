import { getDatabase } from "./schema.js";
import { logWebStoreError, mapOption, uint8ArrayToBase64 } from "./utils.js";

const IDS_FILTER_PREFIX = "Ids:";
const EXPIRED_BEFORE_FILTER_PREFIX = "ExpiredPending:";
const STATUS_PENDING_VARIANT = 0;
const STATUS_COMMITTED_VARIANT = 1;
const STATUS_DISCARDED_VARIANT = 2;

export async function getTransactions(dbId, filter) {
  let transactionRecords = [];
  try {
    const db = getDatabase(dbId);

    if (filter === "Uncommitted") {
      transactionRecords = db
        .prepare("SELECT * FROM transactions WHERE statusVariant = ?")
        .all(STATUS_PENDING_VARIANT);
    } else if (filter.startsWith(IDS_FILTER_PREFIX)) {
      const idsString = filter.substring(IDS_FILTER_PREFIX.length);
      const ids = idsString.split(",");
      if (ids.length > 0) {
        const placeholders = ids.map(() => "?").join(",");
        transactionRecords = db
          .prepare(
            `SELECT * FROM transactions WHERE id IN (${placeholders})`
          )
          .all(...ids);
      }
    } else if (filter.startsWith(EXPIRED_BEFORE_FILTER_PREFIX)) {
      const blockNumString = filter.substring(
        EXPIRED_BEFORE_FILTER_PREFIX.length
      );
      const blockNum = parseInt(blockNumString);
      transactionRecords = db
        .prepare(
          `SELECT * FROM transactions
           WHERE blockNum < ? AND statusVariant != ? AND statusVariant != ?`
        )
        .all(blockNum, STATUS_COMMITTED_VARIANT, STATUS_DISCARDED_VARIANT);
    } else {
      transactionRecords = db.prepare("SELECT * FROM transactions").all();
    }

    if (transactionRecords.length === 0) return [];

    // Fetch associated scripts
    const scriptRoots = transactionRecords
      .map((r) => r.scriptRoot)
      .filter((r) => r != null);

    const scriptMap = new Map();
    if (scriptRoots.length > 0) {
      const placeholders = scriptRoots.map(() => "?").join(",");
      const scripts = db
        .prepare(
          `SELECT * FROM transaction_scripts WHERE scriptRoot IN (${placeholders})`
        )
        .all(...scriptRoots);
      for (const script of scripts) {
        if (script.txScript) {
          scriptMap.set(script.scriptRoot, script.txScript);
        }
      }
    }

    return transactionRecords.map((record) => {
      let txScriptBase64 = undefined;
      if (record.scriptRoot) {
        const txScript = scriptMap.get(record.scriptRoot);
        if (txScript) {
          txScriptBase64 = uint8ArrayToBase64(txScript);
        }
      }
      return {
        id: record.id,
        details: uint8ArrayToBase64(record.details),
        scriptRoot: record.scriptRoot,
        txScript: txScriptBase64,
        blockNum: record.blockNum,
        statusVariant: record.statusVariant,
        status: uint8ArrayToBase64(record.status),
      };
    });
  } catch (err) {
    logWebStoreError(err, "Failed to get transactions");
  }
}

export async function insertTransactionScript(dbId, scriptRoot, txScript) {
  try {
    const db = getDatabase(dbId);
    const scriptRootArray = new Uint8Array(scriptRoot);
    const scriptRootBase64 = uint8ArrayToBase64(scriptRootArray);
    db.prepare(
      "INSERT OR REPLACE INTO transaction_scripts (scriptRoot, txScript) VALUES (?, ?)"
    ).run(
      scriptRootBase64,
      mapOption(txScript, (s) => new Uint8Array(s))
    );
  } catch (error) {
    logWebStoreError(error, "Failed to insert transaction script");
  }
}

export async function upsertTransactionRecord(
  dbId,
  transactionId,
  details,
  blockNum,
  statusVariant,
  status,
  scriptRoot
) {
  try {
    const db = getDatabase(dbId);
    db.prepare(
      `INSERT OR REPLACE INTO transactions
       (id, details, scriptRoot, blockNum, statusVariant, status)
       VALUES (?, ?, ?, ?, ?, ?)`
    ).run(
      transactionId,
      details,
      mapOption(scriptRoot, (root) => uint8ArrayToBase64(root)),
      blockNum,
      statusVariant,
      status
    );
  } catch (err) {
    logWebStoreError(err, "Failed to insert proven transaction data");
  }
}
