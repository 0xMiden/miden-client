import Dexie from "dexie";
import * as semver from "semver";
import { logWebStoreError } from "./utils.js";
export const CLIENT_VERSION_SETTING_KEY = "clientVersion";
const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();
export async function openDatabase(clientVersion, db_name) {
    console.log(`Opening database ${db_name} for client version ${clientVersion}...`);
    try {
        initializeDatabase(db_name);
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
let db;
let accountCodes;
let accountStorages;
let storageMapEntries;
let accountAssets;
let accountAuths;
let accounts;
let addresses;
let transactions;
let transactionScripts;
let inputNotes;
let outputNotes;
let notesScripts;
let stateSync;
let blockHeaders;
let partialBlockchainNodes;
let tags;
let foreignAccountCode;
let settings;
let trackedAccounts;
function indexes(...items) {
    return items.join(",");
}
function initializeDatabase(db_name) {
    if (db && db.name === db_name) {
        return;
    } else if (db) {
        db.close();
    }
    db = new Dexie(db_name);
    db.version(1).stores({
        [Table.AccountCode]: indexes("root"),
        [Table.AccountStorage]: indexes("[commitment+slotName]", "commitment"),
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
    accountCodes = db.table(Table.AccountCode);
    accountStorages = db.table(Table.AccountStorage);
    storageMapEntries = db.table(Table.StorageMapEntries);
    accountAssets = db.table(Table.AccountAssets);
    accountAuths = db.table(Table.AccountAuth);
    accounts = db.table(Table.Accounts);
    addresses = db.table(Table.Addresses);
    transactions = db.table(Table.Transactions);
    transactionScripts = db.table(Table.TransactionScripts);
    inputNotes = db.table(Table.InputNotes);
    outputNotes = db.table(Table.OutputNotes);
    notesScripts = db.table(Table.NotesScripts);
    stateSync = db.table(Table.StateSync);
    blockHeaders = db.table(Table.BlockHeaders);
    partialBlockchainNodes = db.table(Table.PartialBlockchainNodes);
    tags = db.table(Table.Tags);
    foreignAccountCode = db.table(Table.ForeignAccountCode);
    settings = db.table(Table.Settings);
    trackedAccounts = db.table(Table.TrackedAccounts);
    db.on("populate", () => {
        // Populate the stateSync table with default values
        stateSync
            .put({ id: 1, blockNum: "0" })
            .catch((err) => logWebStoreError(err, "Failed to populate DB"));
    });
}
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
