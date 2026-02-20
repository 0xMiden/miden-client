import Dexie from "dexie";
import * as semver from "semver";
import { logWebStoreError } from "./utils.js";
export const CLIENT_VERSION_SETTING_KEY = "clientVersion";
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
    Table["LatestAccountStorage"] = "latestAccountStorage";
    Table["HistoricalAccountStorage"] = "historicalAccountStorage";
    Table["LatestAccountAssets"] = "latestAccountAssets";
    Table["HistoricalAccountAssets"] = "historicalAccountAssets";
    Table["LatestStorageMapEntries"] = "latestStorageMapEntries";
    Table["HistoricalStorageMapEntries"] = "historicalStorageMapEntries";
    Table["AccountAuth"] = "accountAuth";
    Table["AccountKeyMapping"] = "accountKeyMapping";
    Table["LatestAccountHeaders"] = "latestAccountHeaders";
    Table["HistoricalAccountHeaders"] = "historicalAccountHeaders";
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
})(Table || (Table = {}));
function indexes(...items) {
    return items.join(",");
}
/** V1 baseline schema. Extracted as a constant because once migrations are enabled, this must
 *  never be modified — all schema changes should go through new version blocks instead. */
const V1_STORES = {
    [Table.AccountCode]: indexes("root"),
    [Table.LatestAccountStorage]: indexes("[accountId+slotName]", "accountId"),
    [Table.HistoricalAccountStorage]: indexes("[accountId+nonce+slotName]", "accountId", "[accountId+nonce]"),
    [Table.LatestStorageMapEntries]: indexes("[accountId+slotName+key]", "accountId", "[accountId+slotName]"),
    [Table.HistoricalStorageMapEntries]: indexes("[accountId+nonce+slotName+key]", "accountId", "[accountId+nonce]"),
    [Table.LatestAccountAssets]: indexes("[accountId+vaultKey]", "accountId", "faucetIdPrefix"),
    [Table.HistoricalAccountAssets]: indexes("[accountId+nonce+vaultKey]", "accountId", "[accountId+nonce]"),
    [Table.AccountAuth]: indexes("pubKeyCommitmentHex"),
    [Table.AccountKeyMapping]: indexes("[accountIdHex+pubKeyCommitmentHex]", "accountIdHex", "pubKeyCommitmentHex"),
    [Table.LatestAccountHeaders]: indexes("&id", "accountCommitment"),
    [Table.HistoricalAccountHeaders]: indexes("&accountCommitment", "id", "[id+nonce]"),
    [Table.Addresses]: indexes("address", "id"),
    [Table.Transactions]: indexes("id", "statusVariant"),
    [Table.TransactionScripts]: indexes("scriptRoot"),
    [Table.InputNotes]: indexes("noteId", "nullifier", "stateDiscriminant"),
    [Table.OutputNotes]: indexes("noteId", "recipientDigest", "stateDiscriminant", "nullifier"),
    [Table.NotesScripts]: indexes("scriptRoot"),
    [Table.StateSync]: indexes("id"),
    [Table.BlockHeaders]: indexes("blockNum", "hasClientNotes"),
    [Table.PartialBlockchainNodes]: indexes("id"),
    [Table.Tags]: indexes("id++", "tag", "sourceNoteId", "sourceAccountId"),
    [Table.ForeignAccountCode]: indexes("accountId"),
    [Table.Settings]: indexes("key"),
};
export class MidenDatabase {
    dexie;
    accountCodes;
    latestAccountStorages;
    historicalAccountStorages;
    latestStorageMapEntries;
    historicalStorageMapEntries;
    latestAccountAssets;
    historicalAccountAssets;
    accountAuths;
    accountKeyMappings;
    latestAccountHeaders;
    historicalAccountHeaders;
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
    constructor(network) {
        this.dexie = new Dexie(network);
        // --- Schema versioning ---
        //
        // NOTE: The migration system is not currently in use. The Miden network
        // resets on every upgrade, so the database is nuked whenever the client
        // version changes (see ensureClientVersion). Once the network stabilizes
        // and data can be preserved across upgrades, the version-change nuke will
        // be removed and migrations will take over.
        //
        // v1 is the baseline schema. To add a migration:
        //   1. Add a .version(N+1).stores({...}).upgrade(tx => {...}) block below.
        //      Only list tables whose indexes changed; Dexie carries forward the rest.
        //   2. Update TypeScript interfaces and the Table enum if needed.
        //   3. Add a migration test in schema.test.ts.
        //   4. Run `yarn build` and `yarn test`.
        //
        // The version number is a simple incrementing integer, not the client semver.
        // Use a comment to note which client version introduced the change.
        //
        // Example — adding a `createdAt` field with an index to accounts:
        //
        //   // v2: Add createdAt to accounts (client v0.7.0)
        //   this.dexie.version(2).stores({
        //       accounts: indexes("&accountCommitment", "id", ..., "createdAt"),
        //   }).upgrade(tx => {
        //       return tx.table("accounts").toCollection().modify(account => {
        //           account.createdAt = 0;
        //       });
        //   });
        //
        // Tips:
        //   - Index-only changes: omit .upgrade(). Dexie creates indexes automatically.
        //   - New table: just include it in .stores(). It starts empty.
        //   - Remove a table: set it to null, e.g. `oldTable: null`.
        //   - Never modify a previous version block. Always add a new one.
        //
        // Note: The `populate` hook (below the version blocks) only fires on
        // first database creation, NOT during upgrades.
        //
        // To enable migrations (stop nuking the DB on version change):
        //   1. Remove the nuke logic in ensureClientVersion (close/delete/open).
        //      Just persist the new version instead.
        //   2. Freeze V1_STORES — never modify it again.
        //   3. Add version(2+) blocks below for all schema changes going forward.
        this.dexie.version(1).stores(V1_STORES);
        this.accountCodes = this.dexie.table(Table.AccountCode);
        this.latestAccountStorages = this.dexie.table(Table.LatestAccountStorage);
        this.historicalAccountStorages = this.dexie.table(Table.HistoricalAccountStorage);
        this.latestStorageMapEntries = this.dexie.table(Table.LatestStorageMapEntries);
        this.historicalStorageMapEntries = this.dexie.table(Table.HistoricalStorageMapEntries);
        this.latestAccountAssets = this.dexie.table(Table.LatestAccountAssets);
        this.historicalAccountAssets = this.dexie.table(Table.HistoricalAccountAssets);
        this.accountAuths = this.dexie.table(Table.AccountAuth);
        this.accountKeyMappings = this.dexie.table(Table.AccountKeyMapping);
        this.latestAccountHeaders = this.dexie.table(Table.LatestAccountHeaders);
        this.historicalAccountHeaders = this.dexie.table(Table.HistoricalAccountHeaders);
        this.addresses = this.dexie.table(Table.Addresses);
        this.transactions = this.dexie.table(Table.Transactions);
        this.transactionScripts = this.dexie.table(Table.TransactionScripts);
        this.inputNotes = this.dexie.table(Table.InputNotes);
        this.outputNotes = this.dexie.table(Table.OutputNotes);
        this.notesScripts = this.dexie.table(Table.NotesScripts);
        this.stateSync = this.dexie.table(Table.StateSync);
        this.blockHeaders = this.dexie.table(Table.BlockHeaders);
        this.partialBlockchainNodes = this.dexie.table(Table.PartialBlockchainNodes);
        this.tags = this.dexie.table(Table.Tags);
        this.foreignAccountCode = this.dexie.table(Table.ForeignAccountCode);
        this.settings = this.dexie.table(Table.Settings);
        this.dexie.on("populate", () => {
            this.stateSync
                .put({ id: 1, blockNum: 0 })
                .catch((err) => logWebStoreError(err, "Failed to populate DB"));
        });
    }
    async open(clientVersion) {
        console.log(`Opening database ${this.dexie.name} for client version ${clientVersion}...`);
        try {
            await this.dexie.open();
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
        this.dexie.close();
        await this.dexie.delete();
        await this.dexie.open();
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
