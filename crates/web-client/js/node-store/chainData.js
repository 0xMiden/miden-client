import { getDatabase } from "./schema.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";

export async function insertBlockHeader(
  dbId,
  blockNum,
  header,
  partialBlockchainPeaks,
  hasClientNotes
) {
  try {
    const db = getDatabase(dbId);
    db.prepare(
      `INSERT OR REPLACE INTO block_headers (blockNum, header, partialBlockchainPeaks, hasClientNotes)
       VALUES (?, ?, ?, ?)`
    ).run(blockNum, header, partialBlockchainPeaks, hasClientNotes.toString());
  } catch (err) {
    logWebStoreError(err);
  }
}

export async function insertPartialBlockchainNodes(dbId, ids, nodes) {
  try {
    const db = getDatabase(dbId);
    if (ids.length !== nodes.length) {
      throw new Error("ids and nodes arrays must be of the same length");
    }
    if (ids.length === 0) return;

    const stmt = db.prepare(
      "INSERT OR REPLACE INTO partial_blockchain_nodes (id, node) VALUES (?, ?)"
    );
    const insertMany = db.transaction(() => {
      for (let i = 0; i < ids.length; i++) {
        stmt.run(Number(ids[i]), nodes[i]);
      }
    });
    insertMany();
  } catch (err) {
    logWebStoreError(err, "Failed to insert partial blockchain nodes");
  }
}

export async function getBlockHeaders(dbId, blockNumbers) {
  try {
    const db = getDatabase(dbId);
    const nums = Array.from(blockNumbers);
    if (nums.length === 0) return [];

    const placeholders = nums.map(() => "?").join(",");
    const rows = db
      .prepare(
        `SELECT * FROM block_headers WHERE blockNum IN (${placeholders})`
      )
      .all(...nums);

    // Build lookup map for order preservation
    const map = new Map();
    for (const row of rows) {
      map.set(row.blockNum, row);
    }

    // Return in input order, null for missing
    return nums.map((num) => {
      const result = map.get(num);
      if (!result) return null;
      return {
        blockNum: result.blockNum,
        header: uint8ArrayToBase64(result.header),
        partialBlockchainPeaks: uint8ArrayToBase64(
          result.partialBlockchainPeaks
        ),
        hasClientNotes: result.hasClientNotes === "true",
      };
    });
  } catch (err) {
    logWebStoreError(err, "Failed to get block headers");
  }
}

export async function getTrackedBlockHeaders(dbId) {
  try {
    const db = getDatabase(dbId);
    const rows = db
      .prepare("SELECT * FROM block_headers WHERE hasClientNotes = 'true'")
      .all();

    return rows.map((record) => ({
      blockNum: record.blockNum,
      header: uint8ArrayToBase64(record.header),
      partialBlockchainPeaks: uint8ArrayToBase64(
        record.partialBlockchainPeaks
      ),
      hasClientNotes: true,
    }));
  } catch (err) {
    logWebStoreError(err, "Failed to get tracked block headers");
  }
}

export async function getPartialBlockchainPeaksByBlockNum(dbId, blockNum) {
  try {
    const db = getDatabase(dbId);
    const record = db
      .prepare("SELECT partialBlockchainPeaks FROM block_headers WHERE blockNum = ?")
      .get(blockNum);

    if (!record) {
      return { peaks: undefined };
    }

    return {
      peaks: uint8ArrayToBase64(record.partialBlockchainPeaks),
    };
  } catch (err) {
    logWebStoreError(err, "Failed to get partial blockchain peaks");
  }
}

export async function getPartialBlockchainNodesAll(dbId) {
  try {
    const db = getDatabase(dbId);
    return db.prepare("SELECT * FROM partial_blockchain_nodes").all();
  } catch (err) {
    logWebStoreError(err, "Failed to get partial blockchain nodes");
  }
}

export async function getPartialBlockchainNodes(dbId, ids) {
  try {
    const db = getDatabase(dbId);
    const numericIds = Array.from(ids).map((id) => Number(id));
    if (numericIds.length === 0) return [];

    const placeholders = numericIds.map(() => "?").join(",");
    const rows = db
      .prepare(
        `SELECT * FROM partial_blockchain_nodes WHERE id IN (${placeholders})`
      )
      .all(...numericIds);

    // Build lookup map for order preservation
    const map = new Map();
    for (const row of rows) {
      map.set(row.id, row);
    }

    return numericIds.map((id) => map.get(id));
  } catch (err) {
    logWebStoreError(err, "Failed to get partial blockchain nodes");
  }
}

export async function getPartialBlockchainNodesUpToInOrderIndex(
  dbId,
  maxInOrderIndex
) {
  try {
    const db = getDatabase(dbId);
    const maxNumericId = Number(maxInOrderIndex);
    return db
      .prepare("SELECT * FROM partial_blockchain_nodes WHERE id <= ?")
      .all(maxNumericId);
  } catch (err) {
    logWebStoreError(
      err,
      "Failed to get partial blockchain nodes up to index"
    );
  }
}

export async function pruneIrrelevantBlocks(dbId) {
  try {
    const db = getDatabase(dbId);
    db.prepare(
      `DELETE FROM block_headers
       WHERE hasClientNotes = 'false'
         AND blockNum != 0
         AND blockNum != (SELECT blockNum FROM state_sync WHERE id = 1)`
    ).run();
  } catch (err) {
    logWebStoreError(err, "Failed to prune irrelevant blocks");
  }
}
