import Dexie from "dexie";
import { logWebStoreError } from "./utils.js";
const DATABASE_NAME = "MidenClientDB";
export async function openDatabase() {
    console.log("Opening database...");
    try {
        await db.open();
        console.log("Database opened successfully");
        return true;
    }
    catch (err) {
        logWebStoreError(err, "Failed to open database");
        return false;
    }
}
var Table;
(function (Table) {
    Table["AccountCode"] = "accountCode";
    Table["AccountStorage"] = "accountStorage";
    Table["AccountAssets"] = "accountAssets";
    Table["StorageMapEntries"] = "storageMapEntries";
    Table["AccountAuth"] = "accountAuth";
    Table["Accounts"] = "accounts";
    Table["Transactions"] = "transactions";
    Table["TransactionScripts"] = "transactionScripts";
    Table["InputNotes"] = "inputNotes";
    Table["OutputNotes"] = "outputNotes";
    Table["NotesScripts"] = "notesScripts";
    Table["StateSync"] = "stateSync";
    Table["TransportLayerCursor"] = "transportLayerCursor";
    Table["BlockHeaders"] = "blockHeaders";
    Table["PartialBlockchainNodes"] = "partialBlockchainNodes";
    Table["Tags"] = "tags";
    Table["ForeignAccountCode"] = "foreignAccountCode";
    Table["Settings"] = "settings";
})(Table || (Table = {}));
const db = new Dexie(DATABASE_NAME);
db.version(1).stores({
    [Table.AccountCode]: indexes("root"),
    [Table.AccountStorage]: indexes("[commitment+slotIndex]", "commitment"),
    [Table.StorageMapEntries]: indexes("[root+key]", "root"),
    [Table.AccountAssets]: indexes("[root+vaultKey]", "root", "faucetIdPrefix"),
    [Table.AccountAuth]: indexes("pubKey"),
    [Table.Accounts]: indexes("&accountCommitment", "id", "codeRoot", "storageRoot", "vaultRoot"),
    [Table.Transactions]: indexes("id"),
    [Table.TransactionScripts]: indexes("scriptRoot"),
    [Table.InputNotes]: indexes("noteId", "nullifier", "stateDiscriminant"),
    [Table.OutputNotes]: indexes("noteId", "recipientDigest", "stateDiscriminant", "nullifier"),
    [Table.NotesScripts]: indexes("scriptRoot"),
    [Table.StateSync]: indexes("id"),
    [Table.TransportLayerCursor]: indexes("id"),
    [Table.BlockHeaders]: indexes("blockNum", "hasClientNotes"),
    [Table.PartialBlockchainNodes]: indexes("id"),
    [Table.Tags]: indexes("id++", "tag", "source_note_id", "source_account_id"),
    [Table.ForeignAccountCode]: indexes("accountId"),
    [Table.Settings]: indexes("key"),
});
function indexes(...items) {
    return items.join(",");
}
db.on("populate", () => {
    // Populate the stateSync table with default values
    stateSync
        .put({ id: 1, blockNum: "0" })
        .catch((err) => logWebStoreError(err, "Failed to populate DB"));
});
const accountCodes = db.table(Table.AccountCode);
const accountStorages = db.table(Table.AccountStorage);
const storageMapEntries = db.table(Table.StorageMapEntries);
const accountAssets = db.table(Table.AccountAssets);
const accountAuths = db.table(Table.AccountAuth);
const accounts = db.table(Table.Accounts);
const transactions = db.table(Table.Transactions);
const transactionScripts = db.table(Table.TransactionScripts);
const inputNotes = db.table(Table.InputNotes);
const outputNotes = db.table(Table.OutputNotes);
const notesScripts = db.table(Table.NotesScripts);
const stateSync = db.table(Table.StateSync);
const transportLayerCursor = db.table(Table.TransportLayerCursor);
const blockHeaders = db.table(Table.BlockHeaders);
const partialBlockchainNodes = db.table(Table.PartialBlockchainNodes);
const tags = db.table(Table.Tags);
const foreignAccountCode = db.table(Table.ForeignAccountCode);
const settings = db.table(Table.Settings);
export { db, accountCodes, accountStorages, storageMapEntries, accountAssets, accountAuths, accounts, transactions, transactionScripts, inputNotes, outputNotes, notesScripts, stateSync, transportLayerCursor, blockHeaders, partialBlockchainNodes, tags, foreignAccountCode, settings, };
//# sourceMappingURL=schema.js.map
