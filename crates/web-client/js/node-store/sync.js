import { getDatabase } from "./schema.js";
import { logWebStoreError, uint8ArrayToBase64 } from "./utils.js";

export async function getNoteTags(dbId) {
  try {
    const db = getDatabase(dbId);
    const rows = db.prepare("SELECT * FROM tags").all();
    return rows.map((record) => ({
      tag: record.tag,
      sourceNoteId:
        record.source_note_id === "" ? undefined : record.source_note_id,
      sourceAccountId:
        record.source_account_id === ""
          ? undefined
          : record.source_account_id,
    }));
  } catch (error) {
    logWebStoreError(error, "Error fetch tag record");
  }
}

export async function getSyncHeight(dbId) {
  try {
    const db = getDatabase(dbId);
    const record = db
      .prepare("SELECT blockNum FROM state_sync WHERE id = 1")
      .get();
    if (record) {
      return { blockNum: record.blockNum };
    }
    return null;
  } catch (error) {
    logWebStoreError(error, "Error fetching sync height");
  }
}

export async function addNoteTag(dbId, tag, sourceNoteId, sourceAccountId) {
  try {
    const db = getDatabase(dbId);
    const tagArray = new Uint8Array(tag);
    const tagBase64 = uint8ArrayToBase64(tagArray);
    db.prepare(
      "INSERT INTO tags (tag, source_note_id, source_account_id) VALUES (?, ?, ?)"
    ).run(tagBase64, sourceNoteId || "", sourceAccountId || "");
  } catch (error) {
    logWebStoreError(error, "Failed to add note tag");
  }
}

export async function removeNoteTag(dbId, tag, sourceNoteId, sourceAccountId) {
  try {
    const db = getDatabase(dbId);
    const tagArray = new Uint8Array(tag);
    const tagBase64 = uint8ArrayToBase64(tagArray);
    const result = db
      .prepare(
        "DELETE FROM tags WHERE tag = ? AND source_note_id = ? AND source_account_id = ?"
      )
      .run(tagBase64, sourceNoteId || "", sourceAccountId || "");
    return result.changes;
  } catch (error) {
    logWebStoreError(error, "Failed to remove note tag");
  }
}

export async function applyStateSync(dbId, stateUpdate) {
  const db = getDatabase(dbId);
  const {
    blockNum,
    flattenedNewBlockHeaders,
    flattenedPartialBlockChainPeaks,
    newBlockNums,
    blockHasRelevantNotes,
    serializedNodeIds,
    serializedNodes,
    committedNoteIds,
    serializedInputNotes,
    serializedOutputNotes,
    accountUpdates,
    transactionUpdates,
  } = stateUpdate;

  const newBlockHeaders = reconstructFlattenedVec(flattenedNewBlockHeaders);
  const partialBlockchainPeaks = reconstructFlattenedVec(
    flattenedPartialBlockChainPeaks
  );

  // Prepare all statements
  const stmts = {
    getSyncHeight: db.prepare(
      "SELECT blockNum FROM state_sync WHERE id = 1"
    ),
    updateSyncHeight: db.prepare(
      "UPDATE state_sync SET blockNum = ? WHERE id = 1"
    ),
    upsertInputNote: db.prepare(
      `INSERT OR REPLACE INTO input_notes
       (noteId, assets, serialNumber, inputs, scriptRoot, nullifier, stateDiscriminant, state, serializedCreatedAt)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)`
    ),
    upsertNoteScript: db.prepare(
      "INSERT OR REPLACE INTO notes_scripts (scriptRoot, serializedNoteScript) VALUES (?, ?)"
    ),
    upsertOutputNote: db.prepare(
      `INSERT OR REPLACE INTO output_notes
       (noteId, assets, recipientDigest, metadata, nullifier, expectedHeight, stateDiscriminant, state)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?)`
    ),
    upsertTx: db.prepare(
      `INSERT OR REPLACE INTO transactions
       (id, details, blockNum, statusVariant, status, scriptRoot)
       VALUES (?, ?, ?, ?, ?, ?)`
    ),
    upsertTxScript: db.prepare(
      "INSERT OR REPLACE INTO transaction_scripts (scriptRoot, txScript) VALUES (?, ?)"
    ),
    upsertAccountStorage: db.prepare(
      `INSERT OR REPLACE INTO account_storage
       (commitment, slotName, slotValue, slotType) VALUES (?, ?, ?, ?)`
    ),
    upsertStorageMap: db.prepare(
      "INSERT OR REPLACE INTO storage_map_entries (root, key, value) VALUES (?, ?, ?)"
    ),
    upsertVaultAsset: db.prepare(
      `INSERT OR REPLACE INTO account_assets
       (root, vaultKey, faucetIdPrefix, asset) VALUES (?, ?, ?, ?)`
    ),
    upsertAccount: db.prepare(
      `INSERT OR REPLACE INTO accounts
       (id, codeRoot, storageRoot, vaultRoot, nonce, committed, accountCommitment, accountSeed, locked)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0)`
    ),
    upsertTracked: db.prepare(
      "INSERT OR REPLACE INTO tracked_accounts (id) VALUES (?)"
    ),
    getBlockHeader: db.prepare(
      "SELECT blockNum FROM block_headers WHERE blockNum = ?"
    ),
    insertBlockHeader: db.prepare(
      `INSERT INTO block_headers
       (blockNum, header, partialBlockchainPeaks, hasClientNotes)
       VALUES (?, ?, ?, ?)`
    ),
    upsertNode: db.prepare(
      "INSERT OR REPLACE INTO partial_blockchain_nodes (id, node) VALUES (?, ?)"
    ),
    deleteTagByNoteId: db.prepare(
      "DELETE FROM tags WHERE source_note_id = ?"
    ),
  };

  // Execute everything in one atomic transaction
  const applySync = db.transaction(() => {
    // 1. Update sync height (only forward)
    const current = stmts.getSyncHeight.get();
    if (!current || current.blockNum < blockNum) {
      stmts.updateSyncHeight.run(blockNum);
    }

    // 2. Upsert input notes
    for (const note of serializedInputNotes) {
      stmts.upsertInputNote.run(
        note.noteId,
        note.noteAssets,
        note.serialNumber,
        note.inputs,
        note.noteScriptRoot,
        note.nullifier,
        note.stateDiscriminant,
        note.state,
        note.createdAt
      );
      stmts.upsertNoteScript.run(note.noteScriptRoot, note.noteScript);
    }

    // 3. Upsert output notes
    for (const note of serializedOutputNotes) {
      stmts.upsertOutputNote.run(
        note.noteId,
        note.noteAssets,
        note.recipientDigest,
        note.metadata,
        note.nullifier || null,
        note.expectedHeight,
        note.stateDiscriminant,
        note.state
      );
    }

    // 4. Upsert transactions + scripts
    for (const tx of transactionUpdates) {
      const scriptRootBase64 = tx.scriptRoot
        ? uint8ArrayToBase64(tx.scriptRoot)
        : null;
      stmts.upsertTx.run(
        tx.id,
        tx.details,
        tx.blockNum,
        tx.statusVariant,
        tx.status,
        scriptRootBase64
      );
      if (tx.scriptRoot && tx.txScript) {
        stmts.upsertTxScript.run(scriptRootBase64, tx.txScript);
      }
    }

    // 5. Upsert account updates
    for (const acct of accountUpdates) {
      for (const slot of acct.storageSlots) {
        stmts.upsertAccountStorage.run(
          slot.commitment,
          slot.slotName,
          slot.slotValue,
          slot.slotType
        );
      }
      for (const entry of acct.storageMapEntries) {
        stmts.upsertStorageMap.run(entry.root, entry.key, entry.value);
      }
      for (const asset of acct.assets) {
        stmts.upsertVaultAsset.run(
          asset.root,
          asset.vaultKey,
          asset.faucetIdPrefix,
          asset.asset
        );
      }
      stmts.upsertAccount.run(
        acct.accountId,
        acct.codeRoot,
        acct.storageRoot,
        acct.assetVaultRoot,
        acct.nonce,
        acct.committed ? 1 : 0,
        acct.accountCommitment,
        acct.accountSeed || null
      );
      stmts.upsertTracked.run(acct.accountId);
    }

    // 6. Insert block headers (skip if already exists)
    for (let i = 0; i < newBlockHeaders.length; i++) {
      if (!stmts.getBlockHeader.get(newBlockNums[i])) {
        stmts.insertBlockHeader.run(
          newBlockNums[i],
          newBlockHeaders[i],
          partialBlockchainPeaks[i],
          (blockHasRelevantNotes[i] === 1).toString()
        );
      }
    }

    // 7. Insert/update partial blockchain nodes
    for (let i = 0; i < serializedNodeIds.length; i++) {
      stmts.upsertNode.run(
        Number(serializedNodeIds[i]),
        serializedNodes[i]
      );
    }

    // 8. Delete committed note tags
    for (const noteId of committedNoteIds) {
      stmts.deleteTagByNoteId.run(noteId);
    }
  });

  applySync();
}

export async function discardTransactions(dbId, transactions) {
  try {
    const db = getDatabase(dbId);
    const txs = Array.from(transactions);
    if (txs.length === 0) return;
    const placeholders = txs.map(() => "?").join(",");
    db.prepare(
      `DELETE FROM transactions WHERE id IN (${placeholders})`
    ).run(...txs);
  } catch (err) {
    logWebStoreError(err, "Failed to discard transactions");
  }
}

function reconstructFlattenedVec(flattenedVec) {
  const data = flattenedVec.data();
  const lengths = flattenedVec.lengths();
  let index = 0;
  const result = [];
  lengths.forEach((length) => {
    result.push(data.slice(index, index + length));
    index += length;
  });
  return result;
}
