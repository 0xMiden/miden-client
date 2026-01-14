import Dexie from "dexie";
import * as semver from "semver";
import { logWebStoreError } from "./utils.js";
export const CLIENT_VERSION_SETTING_KEY = "clientVersion";
const DATABASE_NAME_PREFIX = "MidenClientDB";
const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();
// Since we can't have a pointer to a JS Object from rust, we'll
// use this instead to keep track of open DBs. A client can have
// a DB for mainnet, devnet, testnet or a custom one, so this should be ok.
const databaseRegistry = new Map();
/**
 * Get a database instance from the registry by its ID.
 * Throws if the database hasn't been opened yet.
 */
export function getDatabase(dbId) {
    const db = databaseRegistry.get(dbId);
    if (!db) {
        throw new Error(`Database not found for id: ${dbId}. Call openDatabase first.`);
    }
    return db;
}
/**
 * Opens a database for the given network and registers it in the registry.
 * Returns the database ID (network name) which can be used to retrieve the database later.
 */
export async function openDatabase(network, clientVersion) {
    const db = new MidenDatabase(network);
    await db.open(clientVersion);
    databaseRegistry.set(network, db);
    return network;
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
function indexes(...items) {
    return items.join(",");
}
export class MidenDatabase {
    db;
    accountCodes;
    accountStorages;
    storageMapEntries;
    accountAssets;
    accountAuths;
    accounts;
    addresses;
    transactions;
    transactionScripts;
    inputNotes;
    outputNotes;
    notesScripts;
    stateSync;
    blockHeaders;
    partialBlockchainNodes;
    tags;
    foreignAccountCode;
    settings;
    trackedAccounts;
    constructor(network) {
        const dbName = `${DATABASE_NAME_PREFIX}_${network}`;
        this.db = new Dexie(dbName);
        this.db.version(1).stores({
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
        this.accountCodes = this.db.table(Table.AccountCode);
        this.accountStorages = this.db.table(Table.AccountStorage);
        this.storageMapEntries = this.db.table(Table.StorageMapEntries);
        this.accountAssets = this.db.table(Table.AccountAssets);
        this.accountAuths = this.db.table(Table.AccountAuth);
        this.accounts = this.db.table(Table.Accounts);
        this.addresses = this.db.table(Table.Addresses);
        this.transactions = this.db.table(Table.Transactions);
        this.transactionScripts = this.db.table(Table.TransactionScripts);
        this.inputNotes = this.db.table(Table.InputNotes);
        this.outputNotes = this.db.table(Table.OutputNotes);
        this.notesScripts = this.db.table(Table.NotesScripts);
        this.stateSync = this.db.table(Table.StateSync);
        this.blockHeaders = this.db.table(Table.BlockHeaders);
        this.partialBlockchainNodes = this.db.table(Table.PartialBlockchainNodes);
        this.tags = this.db.table(Table.Tags);
        this.foreignAccountCode = this.db.table(Table.ForeignAccountCode);
        this.settings = this.db.table(Table.Settings);
        this.trackedAccounts = this.db.table(Table.TrackedAccounts);
        this.db.on("populate", () => {
            this.stateSync
                .put({ id: 1, blockNum: "0" })
                .catch((err) => logWebStoreError(err, "Failed to populate DB"));
        });
    }
    async open(clientVersion) {
        console.log(`Opening database ${this.db.name} for client version ${clientVersion}...`);
        try {
            await this.db.open();
            await this.ensureClientVersion(clientVersion);
            console.log("Database opened successfully");
            return true;
        }
        catch (err) {
            logWebStoreError(err, "Failed to open database");
            return false;
        }
    }
    async ensureClientVersion(clientVersion) {
        if (!clientVersion) {
            console.warn("openDatabase called without a client version; skipping version enforcement.");
            return;
        }
        const storedVersion = await this.getStoredClientVersion();
        if (!storedVersion) {
            await this.persistClientVersion(clientVersion);
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
                await this.persistClientVersion(clientVersion);
                return;
            }
        }
        else {
            console.warn(`Failed to parse semver (${storedVersion} vs ${clientVersion}), forcing store reset.`);
        }
        console.warn(`IndexedDB client version mismatch (stored=${storedVersion}, expected=${clientVersion}). Resetting store.`);
        this.db.close();
        await this.db.delete();
        await this.db.open();
        await this.persistClientVersion(clientVersion);
    }
    async getStoredClientVersion() {
        const record = await this.settings.get(CLIENT_VERSION_SETTING_KEY);
        if (!record) {
            return null;
        }
        return textDecoder.decode(record.value);
    }
    async persistClientVersion(clientVersion) {
        await this.settings.put({
            key: CLIENT_VERSION_SETTING_KEY,
            value: textEncoder.encode(clientVersion),
        });
    }
}
