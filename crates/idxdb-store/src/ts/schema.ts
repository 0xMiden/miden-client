import Dexie from "dexie";
import * as semver from "semver";
import { logWebStoreError } from "./utils.js";

export const CLIENT_VERSION_SETTING_KEY = "clientVersion";
const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();

// Since we can't have a pointer to a JS Object from rust, we'll
// use this instead to keep track of open DBs. A client can have
// a DB for mainnet, devnet, testnet or a custom one, so this should be ok.
const databaseRegistry = new Map<string, MidenDatabase>();

/**
 * Get a database instance from the registry by its ID.
 * Throws if the database hasn't been opened yet.
 */
export function getDatabase(dbId: string): MidenDatabase {
  const db = databaseRegistry.get(dbId);
  if (!db) {
    throw new Error(
      `Database not found for id: ${dbId}. Call openDatabase first.`
    );
  }
  return db;
}

/**
 * Opens a database for the given network and registers it in the registry.
 * Returns the database ID (network name) which can be used to retrieve the database later.
 */
export async function openDatabase(
  network: string,
  clientVersion: string
): Promise<string> {
  const db = new MidenDatabase(network);
  await db.open(clientVersion);
  databaseRegistry.set(network, db);
  return network;
}

enum Table {
  AccountCode = "accountCode",
  AccountStorage = "accountStorage",
  AccountAssets = "accountAssets",
  StorageMapEntries = "storageMapEntries",
  AccountAuth = "accountAuth",
  AccountKeyMapping = "accountKeyMapping",
  Accounts = "accounts",
  Addresses = "addresses",
  Transactions = "transactions",
  TransactionScripts = "transactionScripts",
  InputNotes = "inputNotes",
  OutputNotes = "outputNotes",
  NotesScripts = "notesScripts",
  StateSync = "stateSync",
  BlockHeaders = "blockHeaders",
  PartialBlockchainNodes = "partialBlockchainNodes",
  Tags = "tags",
  ForeignAccountCode = "foreignAccountCode",
  Settings = "settings",
  TrackedAccounts = "trackedAccounts",
}

export interface IAccountCode {
  root: string;
  code: Uint8Array;
}

export interface IAccountStorage {
  commitment: string;
  slotName: string;
  slotValue: string;
  slotType: number;
}

export interface IStorageMapEntry {
  root: string;
  key: string;
  value: string;
}

export interface IAccountAsset {
  root: string;
  vaultKey: string;
  faucetIdPrefix: string;
  asset: string;
}

export interface IAccountAuth {
  pubKeyCommitmentHex: string;
  secretKeyHex: string;
}

export interface IAccountKeyMapping {
  accountIdHex: string;
  pubKeyCommitmentHex: string;
}

export interface IAccount {
  id: string;
  codeRoot: string;
  storageRoot: string;
  vaultRoot: string;
  nonce: string;
  committed: boolean;
  accountSeed?: Uint8Array;
  accountCommitment: string;
  locked: boolean;
}

export interface IAddress {
  address: Uint8Array;
  id: string;
}

export interface ITransaction {
  id: string;
  details: Uint8Array;
  blockNum: number;
  scriptRoot?: string;
  statusVariant: number;
  status: Uint8Array;
}

export interface ITransactionScript {
  scriptRoot: string;
  txScript?: Uint8Array;
}

export interface IInputNote {
  noteId: string;
  stateDiscriminant: number;
  assets: Uint8Array;
  serialNumber: Uint8Array;
  inputs: Uint8Array;
  scriptRoot: string;
  nullifier: string;
  serializedCreatedAt: string;
  state: Uint8Array;
}

export interface IOutputNote {
  noteId: string;
  recipientDigest: string;
  assets: Uint8Array;
  metadata: Uint8Array;
  stateDiscriminant: number;
  nullifier?: string;
  expectedHeight: number;
  state: Uint8Array;
}

export interface INotesScript {
  scriptRoot: string;
  serializedNoteScript: Uint8Array;
}

export interface IStateSync {
  id: number;
  blockNum: number;
}

export interface IBlockHeader {
  blockNum: number;
  header: Uint8Array;
  partialBlockchainPeaks: Uint8Array;
  hasClientNotes: string;
}

export interface IPartialBlockchainNode {
  id: number;
  node: string;
}

export interface ITag {
  id?: number;
  tag: string;
  sourceNoteId?: string;
  sourceAccountId?: string;
}

export interface IForeignAccountCode {
  accountId: string;
  codeRoot: string;
}

export interface ISetting {
  key: string;
  value: Uint8Array;
}

export interface ITrackedAccount {
  id: string;
}

export interface JsVaultAsset {
  root: string;
  vaultKey: string;
  faucetIdPrefix: string;
  asset: string;
}

export interface JsStorageSlot {
  commitment: string;
  slotName: string;
  slotValue: string;
  slotType: number;
}

export interface JsStorageMapEntry {
  root: string;
  key: string;
  value: string;
}

function indexes(...items: string[]): string {
  return items.join(",");
}

export type MidenDexie = Dexie & {
  accountCodes: Dexie.Table<IAccountCode, string>;
  accountStorages: Dexie.Table<IAccountStorage, string>;
  accountAssets: Dexie.Table<IAccountAsset, string>;
  storageMapEntries: Dexie.Table<IStorageMapEntry, string>;
  accountAuths: Dexie.Table<IAccountAuth, string>;
  accountKeyMappings: Dexie.Table<IAccountKeyMapping, string>;
  accounts: Dexie.Table<IAccount, string>;
  addresses: Dexie.Table<IAddress, string>;
  transactions: Dexie.Table<ITransaction, string>;
  transactionScripts: Dexie.Table<ITransactionScript, string>;
  inputNotes: Dexie.Table<IInputNote, string>;
  outputNotes: Dexie.Table<IOutputNote, string>;
  notesScripts: Dexie.Table<INotesScript, string>;
  stateSync: Dexie.Table<IStateSync, number>;
  blockHeaders: Dexie.Table<IBlockHeader, number>;
  partialBlockchainNodes: Dexie.Table<IPartialBlockchainNode, number>;
  tags: Dexie.Table<ITag, number>;
  foreignAccountCode: Dexie.Table<IForeignAccountCode, string>;
  settings: Dexie.Table<ISetting, string>;
  trackedAccounts: Dexie.Table<ITrackedAccount, string>;
};

/** V1 baseline schema. Extracted as a constant because once migrations are enabled, this must
 *  never be modified — all schema changes should go through new version blocks instead. */
const V1_STORES: Record<string, string> = {
  [Table.AccountCode]: indexes("root"),
  [Table.AccountStorage]: indexes("[commitment+slotName]", "commitment"),
  [Table.StorageMapEntries]: indexes("[root+key]", "root"),
  [Table.AccountAssets]: indexes("[root+vaultKey]", "root", "faucetIdPrefix"),
  [Table.AccountAuth]: indexes("pubKeyCommitmentHex"),
  [Table.AccountKeyMapping]: indexes(
    "[accountIdHex+pubKeyCommitmentHex]",
    "accountIdHex",
    "pubKeyCommitmentHex"
  ),
  [Table.Accounts]: indexes(
    "&accountCommitment",
    "id",
    "[id+nonce]",
    "codeRoot",
    "storageRoot",
    "vaultRoot"
  ),
  [Table.Addresses]: indexes("address", "id"),
  [Table.Transactions]: indexes("id", "statusVariant"),
  [Table.TransactionScripts]: indexes("scriptRoot"),
  [Table.InputNotes]: indexes("noteId", "nullifier", "stateDiscriminant"),
  [Table.OutputNotes]: indexes(
    "noteId",
    "recipientDigest",
    "stateDiscriminant",
    "nullifier"
  ),
  [Table.NotesScripts]: indexes("scriptRoot"),
  [Table.StateSync]: indexes("id"),
  [Table.BlockHeaders]: indexes("blockNum", "hasClientNotes"),
  [Table.PartialBlockchainNodes]: indexes("id"),
  [Table.Tags]: indexes("id++", "tag", "sourceNoteId", "sourceAccountId"),
  [Table.ForeignAccountCode]: indexes("accountId"),
  [Table.Settings]: indexes("key"),
  [Table.TrackedAccounts]: indexes("&id"),
};

export class MidenDatabase {
  dexie: MidenDexie;
  accountCodes: Dexie.Table<IAccountCode, string>;
  accountStorages: Dexie.Table<IAccountStorage, string>;
  storageMapEntries: Dexie.Table<IStorageMapEntry, string>;
  accountAssets: Dexie.Table<IAccountAsset, string>;
  accountAuths: Dexie.Table<IAccountAuth, string>;
  accountKeyMappings: Dexie.Table<IAccountKeyMapping, string>;
  accounts: Dexie.Table<IAccount, string>;
  addresses: Dexie.Table<IAddress, string>;
  transactions: Dexie.Table<ITransaction, string>;
  transactionScripts: Dexie.Table<ITransactionScript, string>;
  inputNotes: Dexie.Table<IInputNote, string>;
  outputNotes: Dexie.Table<IOutputNote, string>;
  notesScripts: Dexie.Table<INotesScript, string>;
  stateSync: Dexie.Table<IStateSync, number>;
  blockHeaders: Dexie.Table<IBlockHeader, number>;
  partialBlockchainNodes: Dexie.Table<IPartialBlockchainNode, number>;
  tags: Dexie.Table<ITag, number>;
  foreignAccountCode: Dexie.Table<IForeignAccountCode, string>;
  settings: Dexie.Table<ISetting, string>;
  trackedAccounts: Dexie.Table<ITrackedAccount, string>;

  constructor(network: string) {
    this.dexie = new Dexie(network) as MidenDexie;

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

    this.accountCodes = this.dexie.table<IAccountCode, string>(
      Table.AccountCode
    );
    this.accountStorages = this.dexie.table<IAccountStorage, string>(
      Table.AccountStorage
    );
    this.storageMapEntries = this.dexie.table<IStorageMapEntry, string>(
      Table.StorageMapEntries
    );
    this.accountAssets = this.dexie.table<IAccountAsset, string>(
      Table.AccountAssets
    );
    this.accountAuths = this.dexie.table<IAccountAuth, string>(
      Table.AccountAuth
    );
    this.accountKeyMappings = this.dexie.table<IAccountKeyMapping, string>(
      Table.AccountKeyMapping
    );
    this.accounts = this.dexie.table<IAccount, string>(Table.Accounts);
    this.addresses = this.dexie.table<IAddress, string>(Table.Addresses);
    this.transactions = this.dexie.table<ITransaction, string>(
      Table.Transactions
    );
    this.transactionScripts = this.dexie.table<ITransactionScript, string>(
      Table.TransactionScripts
    );
    this.inputNotes = this.dexie.table<IInputNote, string>(Table.InputNotes);
    this.outputNotes = this.dexie.table<IOutputNote, string>(Table.OutputNotes);
    this.notesScripts = this.dexie.table<INotesScript, string>(
      Table.NotesScripts
    );
    this.stateSync = this.dexie.table<IStateSync, number>(Table.StateSync);
    this.blockHeaders = this.dexie.table<IBlockHeader, number>(
      Table.BlockHeaders
    );
    this.partialBlockchainNodes = this.dexie.table<
      IPartialBlockchainNode,
      number
    >(Table.PartialBlockchainNodes);
    this.tags = this.dexie.table<ITag, number>(Table.Tags);
    this.foreignAccountCode = this.dexie.table<IForeignAccountCode, string>(
      Table.ForeignAccountCode
    );
    this.settings = this.dexie.table<ISetting, string>(Table.Settings);
    this.trackedAccounts = this.dexie.table<ITrackedAccount, string>(
      Table.TrackedAccounts
    );

    this.dexie.on("populate", () => {
      this.stateSync
        .put({ id: 1, blockNum: 0 } as IStateSync)
        .catch((err: unknown) =>
          logWebStoreError(err, "Failed to populate DB")
        );
    });
  }

  async open(clientVersion: string): Promise<boolean> {
    console.log(
      `Opening database ${this.dexie.name} for client version ${clientVersion}...`
    );
    try {
      await this.dexie.open();
      await this.ensureClientVersion(clientVersion);
      console.log("Database opened successfully");
      return true;
    } catch (err) {
      logWebStoreError(err, "Failed to open database");
      return false;
    }
  }

  private async ensureClientVersion(clientVersion: string): Promise<void> {
    if (!clientVersion) {
      console.warn(
        "openDatabase called without a client version; skipping version enforcement."
      );
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
      const sameMajorMinor =
        parsedCurrent?.major === parsedStored?.major &&
        parsedCurrent?.minor === parsedStored?.minor;
      if (sameMajorMinor || !semver.gt(clientVersion, storedVersion)) {
        await this.persistClientVersion(clientVersion);
        return;
      }
    } else {
      console.warn(
        `Failed to parse semver (${storedVersion} vs ${clientVersion}), forcing store reset.`
      );
    }

    console.warn(
      `IndexedDB client version mismatch (stored=${storedVersion}, expected=${clientVersion}). Resetting store.`
    );
    this.dexie.close();
    await this.dexie.delete();
    await this.dexie.open();
    await this.persistClientVersion(clientVersion);
  }

  private async getStoredClientVersion(): Promise<string | null> {
    const record = await this.settings.get(CLIENT_VERSION_SETTING_KEY);
    if (!record) {
      return null;
    }
    return textDecoder.decode(record.value);
  }

  private async persistClientVersion(clientVersion: string): Promise<void> {
    await this.settings.put({
      key: CLIENT_VERSION_SETTING_KEY,
      value: textEncoder.encode(clientVersion),
    });
  }
}
