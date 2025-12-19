import Dexie from "dexie";
import * as semver from "semver";
import { logWebStoreError } from "./utils.js";
const DATABASE_NAME = "MidenClientDB";
export const CLIENT_VERSION_SETTING_KEY = "clientVersion";
const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();
export async function openDatabase(clientVersion) {
    console.log(`Opening database for client version ${clientVersion}...`);
    try {
        await db.open();
        await ensureClientVersion(clientVersion);
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
    Table["Addresses"] = "addresses";
    Table["Transactions"] = "transactions";
    Table["TransactionScripts"] = "transactionScripts";
    Table["InputNotes"] = "inputNotes";
    Table["OutputNotes"] = "outputNotes";
    Table["NotesScripts"] = "notesScripts";
    Table["StateSync"] = "stateSync";
    Table["BlockHeaders"] = "blockHeaders";
    Table["PartialBlockchainNodes"] = "partialBlockchainNodes";
    Table["Tags"] = "tags";
    Table["ForeignAccountCode"] = "foreignAccountCode";
    Table["Settings"] = "settings";
    Table["TrackedAccounts"] = "trackedAccounts";
})(Table || (Table = {}));
const db = new Dexie(DATABASE_NAME);
db.version(1).stores({
    [Table.AccountCode]: indexes("root"),
    [Table.AccountStorage]: indexes("[commitment+slotIndex]", "commitment"),
    [Table.StorageMapEntries]: indexes("[root+key]", "root"),
    [Table.AccountAssets]: indexes("[root+vaultKey]", "root", "faucetIdPrefix"),
    [Table.AccountAuth]: indexes("pubKey"),
    [Table.Accounts]: indexes("&accountCommitment", "id", "[id+nonce]", "codeRoot", "storageRoot", "vaultRoot"),
    [Table.Addresses]: indexes("address", "id"),
    [Table.Transactions]: indexes("id", "statusVariant"),
    [Table.TransactionScripts]: indexes("scriptRoot"),
    [Table.InputNotes]: indexes("noteId", "nullifier", "stateDiscriminant"),
    [Table.OutputNotes]: indexes("noteId", "recipientDigest", "stateDiscriminant", "nullifier"),
    [Table.NotesScripts]: indexes("scriptRoot"),
    [Table.StateSync]: indexes("id"),
    [Table.BlockHeaders]: indexes("blockNum", "hasClientNotes"),
    [Table.PartialBlockchainNodes]: indexes("id"),
    [Table.Tags]: indexes("id++", "tag", "source_note_id", "source_account_id"),
    [Table.ForeignAccountCode]: indexes("accountId"),
    [Table.Settings]: indexes("key"),
    [Table.TrackedAccounts]: indexes("&id"),
});
db.version(2).stores({
    [Table.AccountCode]: indexes("root"),
    [Table.AccountStorage]: indexes("[commitment+slotIndex]", "[commitment+slotName]", "commitment"),
    [Table.StorageMapEntries]: indexes("[root+key]", "root"),
    [Table.AccountAssets]: indexes("[root+vaultKey]", "root", "faucetIdPrefix"),
    [Table.AccountAuth]: indexes("pubKey"),
    [Table.Accounts]: indexes("&accountCommitment", "id", "[id+nonce]", "codeRoot", "storageRoot", "vaultRoot"),
    [Table.Addresses]: indexes("address", "id"),
    [Table.Transactions]: indexes("id", "statusVariant"),
    [Table.TransactionScripts]: indexes("scriptRoot"),
    [Table.InputNotes]: indexes("noteId", "nullifier", "stateDiscriminant"),
    [Table.OutputNotes]: indexes("noteId", "recipientDigest", "stateDiscriminant", "nullifier"),
    [Table.NotesScripts]: indexes("scriptRoot"),
    [Table.StateSync]: indexes("id"),
    [Table.BlockHeaders]: indexes("blockNum", "hasClientNotes"),
    [Table.PartialBlockchainNodes]: indexes("id"),
    [Table.Tags]: indexes("id++", "tag", "source_note_id", "source_account_id"),
    [Table.ForeignAccountCode]: indexes("accountId"),
    [Table.Settings]: indexes("key"),
    [Table.TrackedAccounts]: indexes("&id"),
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
const addresses = db.table(Table.Addresses);
const transactions = db.table(Table.Transactions);
const transactionScripts = db.table(Table.TransactionScripts);
const inputNotes = db.table(Table.InputNotes);
const outputNotes = db.table(Table.OutputNotes);
const notesScripts = db.table(Table.NotesScripts);
const stateSync = db.table(Table.StateSync);
const blockHeaders = db.table(Table.BlockHeaders);
const partialBlockchainNodes = db.table(Table.PartialBlockchainNodes);
const tags = db.table(Table.Tags);
const foreignAccountCode = db.table(Table.ForeignAccountCode);
const settings = db.table(Table.Settings);
const trackedAccounts = db.table(Table.TrackedAccounts);
async function ensureClientVersion(clientVersion) {
    if (!clientVersion) {
        console.warn("openDatabase called without a client version; skipping version enforcement.");
        return;
    }
    const storedVersion = await getStoredClientVersion();
    if (!storedVersion) {
        await persistClientVersion(clientVersion);
        return;
    }
    if (storedVersion === clientVersion) {
        return;
    }
    const validCurrent = semver.valid(clientVersion);
    const validStored = semver.valid(storedVersion);
    if (validCurrent && validStored) {
        const parsedCurrent = semver.parse(validCurrent);
        const parsedStored = semver.parse(validStored);
        const sameMajorMinor = parsedCurrent?.major === parsedStored?.major &&
            parsedCurrent?.minor === parsedStored?.minor;
        if (sameMajorMinor || !semver.gt(clientVersion, storedVersion)) {
            await persistClientVersion(clientVersion);
            return;
        }
    }
    else {
        console.warn(`Failed to parse semver (${storedVersion} vs ${clientVersion}), forcing store reset.`);
    }
    console.warn(`IndexedDB client version mismatch (stored=${storedVersion}, expected=${clientVersion}). Resetting store.`);
    db.close();
    await db.delete();
    await db.open();
    await persistClientVersion(clientVersion);
}
async function getStoredClientVersion() {
    const record = await settings.get(CLIENT_VERSION_SETTING_KEY);
    if (!record) {
        return null;
    }
    return textDecoder.decode(record.value);
}
async function persistClientVersion(clientVersion) {
    await settings.put({
        key: CLIENT_VERSION_SETTING_KEY,
        value: textEncoder.encode(clientVersion),
    });
}
export { db, accountCodes, accountStorages, storageMapEntries, accountAssets, accountAuths, accounts, addresses, transactions, transactionScripts, inputNotes, outputNotes, notesScripts, stateSync, blockHeaders, partialBlockchainNodes, tags, foreignAccountCode, settings, trackedAccounts, };
