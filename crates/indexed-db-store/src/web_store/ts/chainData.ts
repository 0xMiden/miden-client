import { blockHeaders, partialBlockchainNodes, stateSync } from "./schema.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";

// INSERT FUNCTIONS
export async function insertBlockHeader(
  blockNum: string,
  header: Uint8Array,
  partialBlockchainPeaks: Uint8Array,
  hasClientNotes: boolean
) {
  try {
    const data = {
      blockNum: blockNum,
      header,
      partialBlockchainPeaks,
      hasClientNotes: hasClientNotes.toString(),
    };

    const existingBlockHeader = await blockHeaders.get(blockNum);

    if (!existingBlockHeader) {
      await blockHeaders.add(data);
    } else {
      console.log("Block header already exists, checking for update.");

      // Update the hasClientNotes if the existing value is false
      if (existingBlockHeader.hasClientNotes === "false" && hasClientNotes) {
        await blockHeaders.update(blockNum, {
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
  ids: string[],
  nodes: string[]
) {
  try {
    // Check if the arrays are not of the same length
    if (ids.length !== nodes.length) {
      throw new Error("ids and nodes arrays must be of the same length");
    }

    if (ids.length === 0) {
      return;
    }

    // Create array of objects with id and node
    const data = nodes.map((node, index) => ({
      id: ids[index],
      node: node,
    }));

    // Use bulkPut to add/overwrite the entries
    await partialBlockchainNodes.bulkPut(data);
  } catch (err) {
    logWebStoreError(err, "Failed to insert partial blockchain nodes");
  }
}

// GET FUNCTIONS
export async function getBlockHeaders(blockNumbers: string[]) {
  try {
    const results = await blockHeaders.bulkGet(blockNumbers);

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

export async function getTrackedBlockHeaders() {
  try {
    // Fetch all records matching the given root
    const allMatchingRecords = await blockHeaders
      .where("hasClientNotes")
      .equals("true")
      .toArray();

    // Process all records with async operations
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

export async function getPartialBlockchainPeaksByBlockNum(blockNum: string) {
  try {
    const blockHeader = await blockHeaders.get(blockNum);
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

export async function getPartialBlockchainNodesAll() {
  try {
    const partialBlockchainNodesAll = await partialBlockchainNodes.toArray();
    return partialBlockchainNodesAll;
  } catch (err) {
    logWebStoreError(err, "Failed to get partial blockchain nodes");
  }
}

export async function getPartialBlockchainNodes(ids: string[]) {
  try {
    const results = await partialBlockchainNodes.bulkGet(ids);

    return results;
  } catch (err) {
    logWebStoreError(err, "Failed to get partial blockchain nodes");
  }
}

export async function pruneIrrelevantBlocks() {
  try {
    const syncHeight = await stateSync.get(1);

    if (syncHeight == undefined) {
      throw Error("SyncHeight is undefined -- is the state sync table empty?");
    }

    const allMatchingRecords = await blockHeaders
      .where("hasClientNotes")
      .equals("false")
      .and(
        (record) =>
          record.blockNum !== "0" && record.blockNum !== syncHeight.blockNum
      )
      .toArray();

    await blockHeaders.bulkDelete(allMatchingRecords.map((r) => r.blockNum));
  } catch (err) {
    logWebStoreError(err, "Failed to prune irrelevant blocks");
  }
}
