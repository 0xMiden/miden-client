import { getDatabase } from "./schema.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";

export async function insertBlockHeader(
  dbId: string,
  blockNum: number,
  header: Uint8Array,
  partialBlockchainPeaks: Uint8Array,
  hasClientNotes: boolean
) {
  try {
    const db = getDatabase(dbId);
    const data = {
      blockNum: blockNum,
      header,
      partialBlockchainPeaks,
      hasClientNotes: hasClientNotes.toString(),
    };

    // Mirror SQLite's `insert_block_header_tx`: do an INSERT OR IGNORE on the
    // row, then explicitly upgrade `has_client_notes` to true if the caller
    // says so. Two callers hit this:
    //   - Genesis flow — no existing row; the add succeeds.
    //   - `get_and_store_authenticated_block` for a past block — a row
    //     written by `applyStateSync` typically already exists. Overwriting
    //     it would clobber the correct historical peaks (popcount ==
    //     block_num) with peaks from the caller's current `PartialMmr`
    //     forest (popcount == current sync height). Later reads of those
    //     peaks trip `MmrPeaks::new`'s InvalidPeaks validation and wedge
    //     sync for the rest of the session.
    //
    // The `has_client_notes` upgrade is load-bearing: `get_tracked_block_
    // header_numbers` filters by this flag to seed `tracked_leaves`, which
    // `get_partial_blockchain_nodes(Forest)` relies on. A private-note
    // import at a block previously synced as irrelevant must flip the flag
    // to true or the auth paths won't be tracked.
    await db.blockHeaders.add(data).catch(async (err: unknown) => {
      if (!isConstraintError(err)) throw err;
      if (hasClientNotes) {
        await db.blockHeaders.update(blockNum, { hasClientNotes: "true" });
      }
    });
  } catch (err) {
    logWebStoreError(err);
  }
}

/** Detect Dexie's primary-key collision error across sync + bulk wrappings. */
function isConstraintError(err: unknown): boolean {
  const e = err as
    | { name?: string; inner?: { name?: string } }
    | null
    | undefined;
  return e?.name === "ConstraintError" || e?.inner?.name === "ConstraintError";
}

export async function insertPartialBlockchainNodes(
  dbId: string,
  ids: string[],
  nodes: string[]
) {
  try {
    const db = getDatabase(dbId);
    if (ids.length !== nodes.length) {
      throw new Error("ids and nodes arrays must be of the same length");
    }

    if (ids.length === 0) {
      return;
    }

    const data = nodes.map((node, index) => ({
      id: Number(ids[index]),
      node: node,
    }));

    await db.partialBlockchainNodes.bulkPut(data);
  } catch (err) {
    logWebStoreError(err, "Failed to insert partial blockchain nodes");
  }
}

export async function getBlockHeaders(dbId: string, blockNumbers: number[]) {
  try {
    const db = getDatabase(dbId);
    const results = await db.blockHeaders.bulkGet(blockNumbers);

    const processedResults = await Promise.all(
      results.map((result) => {
        if (result === undefined) {
          return null;
        } else {
          const headerBase64 = uint8ArrayToBase64(result.header);
          const partialBlockchainPeaksBase64 = uint8ArrayToBase64(
            result.partialBlockchainPeaks
          );

          return {
            blockNum: result.blockNum,
            header: headerBase64,
            partialBlockchainPeaks: partialBlockchainPeaksBase64,
            hasClientNotes: result.hasClientNotes === "true",
          };
        }
      })
    );

    return processedResults;
  } catch (err) {
    logWebStoreError(err, "Failed to get block headers");
  }
}

export async function getTrackedBlockHeaders(dbId: string) {
  try {
    const db = getDatabase(dbId);
    const allMatchingRecords = await db.blockHeaders
      .where("hasClientNotes")
      .equals("true")
      .toArray();

    const processedRecords = await Promise.all(
      allMatchingRecords.map((record) => {
        const headerBase64 = uint8ArrayToBase64(record.header);

        const partialBlockchainPeaksBase64 = uint8ArrayToBase64(
          record.partialBlockchainPeaks
        );

        return {
          blockNum: record.blockNum,
          header: headerBase64,
          partialBlockchainPeaks: partialBlockchainPeaksBase64,
          hasClientNotes: record.hasClientNotes === "true",
        };
      })
    );

    return processedRecords;
  } catch (err) {
    logWebStoreError(err, "Failed to get tracked block headers");
  }
}

export async function getTrackedBlockHeaderNumbers(dbId: string) {
  try {
    const db = getDatabase(dbId);
    const blockNums = await db.blockHeaders
      .where("hasClientNotes")
      .equals("true")
      .primaryKeys();
    return blockNums;
  } catch (err) {
    logWebStoreError(err, "Failed to get tracked block header numbers");
  }
}

export async function getPartialBlockchainPeaksByBlockNum(
  dbId: string,
  blockNum: number
) {
  try {
    const db = getDatabase(dbId);
    const blockHeader = await db.blockHeaders.get(blockNum);
    if (blockHeader == undefined) {
      return {
        peaks: undefined,
      };
    }
    const partialBlockchainPeaksBase64 = uint8ArrayToBase64(
      blockHeader.partialBlockchainPeaks
    );

    return {
      peaks: partialBlockchainPeaksBase64,
    };
  } catch (err) {
    logWebStoreError(err, "Failed to get partial blockchain peaks");
  }
}

export async function getPartialBlockchainNodesAll(dbId: string) {
  try {
    const db = getDatabase(dbId);
    const partialBlockchainNodesAll = await db.partialBlockchainNodes.toArray();
    return partialBlockchainNodesAll;
  } catch (err) {
    logWebStoreError(err, "Failed to get partial blockchain nodes");
  }
}

export async function getPartialBlockchainNodes(dbId: string, ids: string[]) {
  try {
    const db = getDatabase(dbId);
    const numericIds = ids.map((id) => Number(id));
    const results = await db.partialBlockchainNodes.bulkGet(numericIds);

    // bulkGet returns undefined for missing keys — filter them out so the
    // Rust deserializer does not choke on undefined values.
    return results.filter((r) => r !== undefined);
  } catch (err) {
    logWebStoreError(err, "Failed to get partial blockchain nodes");
  }
}

export async function getPartialBlockchainNodesUpToInOrderIndex(
  dbId: string,
  maxInOrderIndex: string
) {
  try {
    const db = getDatabase(dbId);
    const maxNumericId = Number(maxInOrderIndex);
    const results = await db.partialBlockchainNodes
      .where("id")
      .belowOrEqual(maxNumericId)
      .toArray();
    return results;
  } catch (err) {
    logWebStoreError(err, "Failed to get partial blockchain nodes up to index");
  }
}

export async function pruneIrrelevantBlocks(
  dbId: string,
  blocksToUntrack: number[],
  nodeIdsToRemove: string[]
) {
  try {
    const db = getDatabase(dbId);
    const numericNodeIds = nodeIdsToRemove.map(Number);

    const syncHeight = await db.stateSync.get(1);
    if (syncHeight == undefined) {
      throw Error("SyncHeight is undefined -- is the state sync table empty?");
    }

    await db.dexie.transaction(
      "rw",
      db.blockHeaders,
      db.partialBlockchainNodes,
      async () => {
        // 1. Delete stale MMR authentication nodes.
        if (numericNodeIds.length > 0) {
          await db.partialBlockchainNodes.bulkDelete(numericNodeIds);
        }

        // 2. Mark untracked blocks as irrelevant.
        if (blocksToUntrack.length > 0) {
          await db.blockHeaders
            .where("blockNum")
            .anyOf(blocksToUntrack)
            .modify({ hasClientNotes: "false" });
        }

        // 3. Delete irrelevant block headers.
        const allMatchingRecords = await db.blockHeaders
          .where("hasClientNotes")
          .equals("false")
          .and(
            (record) =>
              record.blockNum !== 0 && record.blockNum !== syncHeight.blockNum
          )
          .toArray();

        await db.blockHeaders.bulkDelete(
          allMatchingRecords.map((r) => r.blockNum)
        );
      }
    );
  } catch (err) {
    logWebStoreError(err, "Failed to prune irrelevant blocks");
  }
}
