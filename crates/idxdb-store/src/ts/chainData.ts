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

    const existingBlockHeader = await db.blockHeaders.get(blockNum);

    if (!existingBlockHeader) {
      await db.blockHeaders.put(data);
    } else {
      console.log("Block header already exists, checking for update.");

      if (existingBlockHeader.hasClientNotes === "false" && hasClientNotes) {
        await db.blockHeaders.update(blockNum, {
          hasClientNotes: hasClientNotes.toString(),
        });
        console.log("Updated hasClientNotes to true.");
      } else {
        console.log("No update needed for hasClientNotes.");
      }
    }
  } catch (err) {
    logWebStoreError(err);
  }
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

    return results;
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

export async function pruneIrrelevantBlocks(dbId: string) {
  try {
    const db = getDatabase(dbId);
    const syncHeight = await db.stateSync.get(1);

    if (syncHeight == undefined) {
      throw Error("SyncHeight is undefined -- is the state sync table empty?");
    }

    const allMatchingRecords = await db.blockHeaders
      .where("hasClientNotes")
      .equals("false")
      .and(
        (record) =>
          record.blockNum !== 0 && record.blockNum !== syncHeight.blockNum
      )
      .toArray();

    await db.blockHeaders.bulkDelete(allMatchingRecords.map((r) => r.blockNum));
  } catch (err) {
    logWebStoreError(err, "Failed to prune irrelevant blocks");
  }
}
