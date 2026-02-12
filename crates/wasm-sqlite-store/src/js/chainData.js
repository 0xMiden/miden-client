/**
 * Chain data operations for the WASM SQLite store.
 * Handles block headers, partial blockchain nodes, and MMR peaks.
 */
import { getDatabase } from "./schema.js";
import { logError, uint8ArrayToBase64 } from "./utils.js";
function toBase64(data) {
  if (!data) return "";
  const bytes = data instanceof Uint8Array ? data : new Uint8Array(data);
  return uint8ArrayToBase64(bytes);
}
export function getBlockHeaders(dbId, blockNumbers) {
  try {
    if (blockNumbers.length === 0) return [];
    const db = getDatabase(dbId);
    const placeholders = blockNumbers.map(() => "?").join(",");
    const rows = db.all(
      `SELECT block_num, header, partial_blockchain_peaks, has_client_notes
       FROM block_headers
       WHERE block_num IN (${placeholders})`,
      blockNumbers
    );
    return rows.map((row) => ({
      blockNum: row.block_num,
      header: toBase64(row.header),
      partialBlockchainPeaks: toBase64(row.partial_blockchain_peaks),
      hasClientNotes: row.has_client_notes === 1,
    }));
  } catch (error) {
    logError(error, "Error fetching block headers");
    return [];
  }
}
export function getTrackedBlockHeaders(dbId) {
  try {
    const db = getDatabase(dbId);
    const rows =
      db.all(`SELECT block_num, header, partial_blockchain_peaks, has_client_notes
       FROM block_headers
       WHERE has_client_notes = 1`);
    return rows.map((row) => ({
      blockNum: row.block_num,
      header: toBase64(row.header),
      partialBlockchainPeaks: toBase64(row.partial_blockchain_peaks),
      hasClientNotes: true,
    }));
  } catch (error) {
    logError(error, "Error fetching tracked block headers");
    return [];
  }
}
export function getPartialBlockchainNodesAll(dbId) {
  try {
    const db = getDatabase(dbId);
    const rows = db.all("SELECT id, node FROM partial_blockchain_nodes");
    return rows.map((row) => ({
      id: row.id.toString(),
      node: toBase64(row.node),
    }));
  } catch (error) {
    logError(error, "Error fetching all partial blockchain nodes");
    return [];
  }
}
export function getPartialBlockchainNodes(dbId, ids) {
  try {
    if (ids.length === 0) return [];
    const db = getDatabase(dbId);
    const placeholders = ids.map(() => "?").join(",");
    const rows = db.all(
      `SELECT id, node FROM partial_blockchain_nodes WHERE id IN (${placeholders})`,
      ids.map((id) => parseInt(id))
    );
    return rows.map((row) => ({
      id: row.id.toString(),
      node: toBase64(row.node),
    }));
  } catch (error) {
    logError(error, "Error fetching partial blockchain nodes");
    return [];
  }
}
export function getPartialBlockchainNodesUpToInOrderIndex(
  dbId,
  maxInOrderIndex
) {
  try {
    const db = getDatabase(dbId);
    const rows = db.all(
      "SELECT id, node FROM partial_blockchain_nodes WHERE id <= ?",
      [parseInt(maxInOrderIndex)]
    );
    return rows.map((row) => ({
      id: row.id.toString(),
      node: toBase64(row.node),
    }));
  } catch (error) {
    logError(error, "Error fetching partial blockchain nodes up to index");
    return [];
  }
}
export function getPartialBlockchainPeaksByBlockNum(dbId, blockNum) {
  try {
    const db = getDatabase(dbId);
    const row = db.get(
      "SELECT partial_blockchain_peaks FROM block_headers WHERE block_num = ?",
      [blockNum]
    );
    return {
      partialBlockchainPeaks: row
        ? toBase64(row.partial_blockchain_peaks)
        : null,
    };
  } catch (error) {
    logError(error, `Error fetching peaks for block ${blockNum}`);
    return { partialBlockchainPeaks: null };
  }
}
export function insertBlockHeader(
  dbId,
  blockNum,
  header,
  partialBlockchainPeaks,
  hasClientNotes
) {
  try {
    const db = getDatabase(dbId);
    // Use INSERT OR REPLACE to handle the case where we want to update has_client_notes
    // If the block exists and has_client_notes is being set to true, update it
    const existing = db.get(
      "SELECT has_client_notes FROM block_headers WHERE block_num = ?",
      [blockNum]
    );
    if (existing) {
      if (hasClientNotes && !existing.has_client_notes) {
        db.run(
          "UPDATE block_headers SET has_client_notes = 1 WHERE block_num = ?",
          [blockNum]
        );
      }
    } else {
      db.run(
        `INSERT INTO block_headers (block_num, header, partial_blockchain_peaks, has_client_notes)
         VALUES (?, ?, ?, ?)`,
        [blockNum, header, partialBlockchainPeaks, hasClientNotes ? 1 : 0]
      );
    }
  } catch (error) {
    logError(error, `Error inserting block header: ${blockNum}`);
  }
}
export function insertPartialBlockchainNodes(dbId, ids, nodes) {
  try {
    const db = getDatabase(dbId);
    db.transaction(() => {
      for (let i = 0; i < ids.length; i++) {
        db.run(
          "INSERT OR IGNORE INTO partial_blockchain_nodes (id, node) VALUES (?, ?)",
          [parseInt(ids[i]), nodes[i]]
        );
      }
    });
  } catch (error) {
    logError(error, "Error inserting partial blockchain nodes");
  }
}
export function pruneIrrelevantBlocks(dbId) {
  try {
    const db = getDatabase(dbId);
    // Get the genesis block number (min) and the latest sync block (max)
    const syncRow = db.get(
      "SELECT block_num FROM state_sync ORDER BY block_num DESC LIMIT 1"
    );
    const genesisRow = db.get(
      "SELECT MIN(block_num) as block_num FROM block_headers"
    );
    if (!syncRow || !genesisRow) return;
    // Delete block headers that:
    // 1. Don't have client notes
    // 2. Are not the genesis block
    // 3. Are not the latest sync block
    db.run(
      `DELETE FROM block_headers
       WHERE has_client_notes = 0
       AND block_num != ?
       AND block_num != ?`,
      [genesisRow.block_num, syncRow.block_num]
    );
  } catch (error) {
    logError(error, "Error pruning irrelevant blocks");
  }
}
