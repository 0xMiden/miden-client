import Dexie from "dexie";
import * as semver from "semver";
import { logWebStoreError } from "./utils.js";

const DATABASE_NAME = "MidenClientDB";
export const CLIENT_VERSION_SETTING_KEY = "clientVersion";
const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();

export async function openDatabase(clientVersion: string): Promise<boolean> {
  console.log(`Opening database for client version ${clientVersion}...`);
  try {
    await db.open();
    await ensureClientVersion(clientVersion);
    console.log("Database opened successfully");
    return true;
  } catch (err) {
    logWebStoreError(err, "Failed to open database");
    return false;
  }
}

enum Table {
  AccountCode = "accountCode",
  AccountStorage = "accountStorage",
  AccountAssets = "accountAssets",
  StorageMapEntries = "storageMapEntries",
  AccountAuth = "accountAuth",
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
  pubKey: string;
  secretKey: string;
  accountIdHex: string;
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
  blockNum: string;
}

export interface IBlockHeader {
  blockNum: string;
  header: Uint8Array;
  partialBlockchainPeaks: Uint8Array;
  hasClientNotes: string;
}

export interface IPartialBlockchainNode {
  id: string;
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

const db = new Dexie(DATABASE_NAME) as Dexie & {
  accountCodes: Dexie.Table<IAccountCode, string>;
  accountStorages: Dexie.Table<IAccountStorage, string>;
  accountAssets: Dexie.Table<IAccountAsset, string>;
  storageMapEntries: Dexie.Table<IStorageMapEntry, string>;
  accountAuths: Dexie.Table<IAccountAuth, string>;
  accounts: Dexie.Table<IAccount, string>;
  addresses: Dexie.Table<IAddress, string>;
  transactions: Dexie.Table<ITransaction, string>;
  transactionScripts: Dexie.Table<ITransactionScript, string>;
  inputNotes: Dexie.Table<IInputNote, string>;
  outputNotes: Dexie.Table<IOutputNote, string>;
  notesScripts: Dexie.Table<INotesScript, string>;
  stateSync: Dexie.Table<IStateSync, number>;
  blockHeaders: Dexie.Table<IBlockHeader, string>;
  partialBlockchainNodes: Dexie.Table<IPartialBlockchainNode, string>;
  tags: Dexie.Table<ITag, number>;
  foreignAccountCode: Dexie.Table<IForeignAccountCode, string>;
  settings: Dexie.Table<ISetting, string>;
  trackedAccounts: Dexie.Table<ITrackedAccount, string>;
};

db.version(1).stores({
  [Table.AccountCode]: indexes("root"),
  [Table.AccountStorage]: indexes("[commitment+slotName]", "commitment"),
  [Table.StorageMapEntries]: indexes("[root+key]", "root"),
  [Table.AccountAssets]: indexes("[root+vaultKey]", "root", "faucetIdPrefix"),
  [Table.AccountAuth]: indexes("pubKey"),
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
  [Table.Tags]: indexes("id++", "tag", "source_note_id", "source_account_id"),
  [Table.ForeignAccountCode]: indexes("accountId"),
  [Table.Settings]: indexes("key"),
  [Table.TrackedAccounts]: indexes("&id"),
});

function indexes(...items: string[]): string {
  return items.join(",");
}

db.on("populate", () => {
  // Populate the stateSync table with default values
  stateSync
    .put({ id: 1, blockNum: "0" } as IStateSync)
    .catch((err: unknown) => logWebStoreError(err, "Failed to populate DB"));
});

const accountCodes = db.table<IAccountCode, string>(Table.AccountCode);
const accountStorages = db.table<IAccountStorage, string>(Table.AccountStorage);
const storageMapEntries = db.table<IStorageMapEntry, string>(
  Table.StorageMapEntries
);
const accountAssets = db.table<IAccountAsset, string>(Table.AccountAssets);
const accountAuths = db.table<IAccountAuth, string>(Table.AccountAuth);
const accounts = db.table<IAccount, string>(Table.Accounts);
const addresses = db.table<IAddress, string>(Table.Addresses);
const transactions = db.table<ITransaction, string>(Table.Transactions);
const transactionScripts = db.table<ITransactionScript, string>(
  Table.TransactionScripts
);
const inputNotes = db.table<IInputNote, string>(Table.InputNotes);
const outputNotes = db.table<IOutputNote, string>(Table.OutputNotes);
const notesScripts = db.table<INotesScript, string>(Table.NotesScripts);
const stateSync = db.table<IStateSync, number>(Table.StateSync);
const blockHeaders = db.table<IBlockHeader, string>(Table.BlockHeaders);
const partialBlockchainNodes = db.table<IPartialBlockchainNode, string>(
  Table.PartialBlockchainNodes
);
const tags = db.table<ITag, number>(Table.Tags);
const foreignAccountCode = db.table<IForeignAccountCode, string>(
  Table.ForeignAccountCode
);
const settings = db.table<ISetting, string>(Table.Settings);
const trackedAccounts = db.table<ITrackedAccount, string>(
  Table.TrackedAccounts
);

async function ensureClientVersion(clientVersion: string): Promise<void> {
  if (!clientVersion) {
    console.warn(
      "openDatabase called without a client version; skipping version enforcement."
    );
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
    const sameMajorMinor =
      parsedCurrent?.major === parsedStored?.major &&
      parsedCurrent?.minor === parsedStored?.minor;
    if (sameMajorMinor || !semver.gt(clientVersion, storedVersion)) {
      await persistClientVersion(clientVersion);
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
  db.close();
  await db.delete();
  await db.open();
  await persistClientVersion(clientVersion);
}

async function getStoredClientVersion(): Promise<string | null> {
  const record = await settings.get(CLIENT_VERSION_SETTING_KEY);
  if (!record) {
    return null;
  }
  return textDecoder.decode(record.value);
}

async function persistClientVersion(clientVersion: string): Promise<void> {
  await settings.put({
    key: CLIENT_VERSION_SETTING_KEY,
    value: textEncoder.encode(clientVersion),
  });
}

export {
  db,
  accountCodes,
  accountStorages,
  storageMapEntries,
  accountAssets,
  accountAuths,
  accounts,
  addresses,
  transactions,
  transactionScripts,
  inputNotes,
  outputNotes,
  notesScripts,
  stateSync,
  blockHeaders,
  partialBlockchainNodes,
  tags,
  foreignAccountCode,
  settings,
  trackedAccounts,
};
