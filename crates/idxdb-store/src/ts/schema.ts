import Dexie from "dexie";
import * as semver from "semver";
import { isNodeRuntime, logWebStoreError } from "./utils.js";

const DATABASE_NAME = "MidenClientDB";
export const CLIENT_VERSION_SETTING_KEY = "clientVersion";
const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();
const runningInNode = isNodeRuntime();
let nodeIndexedDbPromise: Promise<void> | null = null;
let shimReady: Promise<void> | null = null;
let nodeIndexedDbBasePath: string | null = null;

export async function openDatabase(clientVersion: string): Promise<boolean> {
  console.log(`Opening database for client version ${clientVersion}...`);
  try {
    if (runningInNode && !shimReady) {
      shimReady = ensureNodeIndexedDbShim();
    }
    await shimReady;
    if (runningInNode) {
      reinitializeDbForNode();
    }
    await db.open();
    await ensureClientVersion(clientVersion);
    console.log("Database opened successfully");
    return true;
  } catch (err) {
    logWebStoreError(err, "Failed to open database");
    return false;
  }
}

async function loadNodeIndexedDbShim() {
  const nodeGlobal = globalThis as typeof globalThis & {
    require?: NodeRequire;
  };
  if (typeof nodeGlobal.require === "function") {
    const shimModule = nodeGlobal.require(
      "indexeddbshim/dist/indexeddbshim-node.js"
    );
    return shimModule.default ?? shimModule;
  }
  const { createRequire } = await import("node:module");
  const require = createRequire(import.meta.url);
  const shimModule = require("indexeddbshim/dist/indexeddbshim-node.js");
  return shimModule.default ?? shimModule;
}

async function ensureNodeIndexedDbShim(): Promise<void> {
  if (!runningInNode) {
    return;
  }
  if (!nodeIndexedDbPromise) {
    nodeIndexedDbPromise = (async () => {
      try {
        const globals = globalThis as typeof globalThis & {
          window?: Window & typeof globalThis;
          navigator?: Navigator;
          self?: Window & typeof globalThis;
          indexedDB?: IDBFactory;
          IDBKeyRange?: typeof IDBKeyRange;
        };
        if (!globals.window) {
          globals.window = globals as unknown as Window & typeof globalThis;
        }
        if (!globals.self) {
          globals.self = globals.window;
        }
        if (!globals.navigator) {
          globals.navigator = { userAgent: "node" } as Navigator;
        }
        const [setGlobalVars, pathModule, fsPromises] = await Promise.all([
          loadNodeIndexedDbShim(),
          import("node:path"),
          import("node:fs/promises"),
        ]);
        const resolveBasePath = () => {
          const configuredPath = process.env.MIDEN_NODE_INDEXEDDB_BASE_PATH;
          if (configuredPath && configuredPath.trim().length > 0) {
            return pathModule.resolve(configuredPath);
          }
          return pathModule.join(process.cwd(), "miden-webclient-indexeddb");
        };
        const databaseBasePath = resolveBasePath();
        await fsPromises.mkdir(databaseBasePath, { recursive: true });
        nodeIndexedDbBasePath = databaseBasePath;
        const shimmed = setGlobalVars(globals, {
          checkOrigin: false,
          addNonIDBGlobals: true,
          databaseBasePath,
        });
       if (!globals.indexedDB) {
         globals.indexedDB = shimmed.indexedDB;
       }
       if (!globals.IDBKeyRange) {
         globals.IDBKeyRange = shimmed.IDBKeyRange;
       }
        if (globals.indexedDB) {
          Dexie.dependencies.indexedDB = globals.indexedDB;
        }
        if (globals.IDBKeyRange) {
          Dexie.dependencies.IDBKeyRange = globals.IDBKeyRange;
        }
      } catch (error) {
        console.error("Failed to initialize IndexedDB shim for Node.js", error);
        throw error;
      }
    })();
  }
  return nodeIndexedDbPromise;
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
  slotIndex: number;
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

if (runningInNode) {
  const globalWithIDB = globalThis as typeof globalThis & {
    indexedDB?: IDBFactory;
    IDBKeyRange?: typeof IDBKeyRange;
  };
  if (globalWithIDB.indexedDB) {
    Dexie.dependencies.indexedDB = globalWithIDB.indexedDB;
  }
  if (globalWithIDB.IDBKeyRange) {
    Dexie.dependencies.IDBKeyRange = globalWithIDB.IDBKeyRange;
  }
}

let db = new Dexie(DATABASE_NAME);

const configureDatabase = (instance: Dexie) => {
  instance.version(1).stores({
    [Table.AccountCode]: indexes("root"),
    [Table.AccountStorage]: indexes("[commitment+slotIndex]", "commitment"),
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
    [Table.Tags]: indexes(
      "id++",
      "tag",
      "source_note_id",
      "source_account_id"
    ),
    [Table.ForeignAccountCode]: indexes("accountId"),
    [Table.Settings]: indexes("key"),
    [Table.TrackedAccounts]: indexes("&id"),
  });
};

let accountCodes: Dexie.Table<IAccountCode, string>;
let accountStorages: Dexie.Table<IAccountStorage, string>;
let storageMapEntries: Dexie.Table<IStorageMapEntry, string>;
let accountAssets: Dexie.Table<IAccountAsset, string>;
let accountAuths: Dexie.Table<IAccountAuth, string>;
let accounts: Dexie.Table<IAccount, string>;
let addresses: Dexie.Table<IAddress, string>;
let transactions: Dexie.Table<ITransaction, string>;
let transactionScripts: Dexie.Table<ITransactionScript, string>;
let inputNotes: Dexie.Table<IInputNote, string>;
let outputNotes: Dexie.Table<IOutputNote, string>;
let notesScripts: Dexie.Table<INotesScript, string>;
let stateSync: Dexie.Table<IStateSync, number>;
let blockHeaders: Dexie.Table<IBlockHeader, string>;
let partialBlockchainNodes: Dexie.Table<IPartialBlockchainNode, string>;
let tags: Dexie.Table<ITag, number>;
let foreignAccountCode: Dexie.Table<IForeignAccountCode, string>;
let settings: Dexie.Table<ISetting, string>;
let trackedAccounts: Dexie.Table<ITrackedAccount, string>;

const bindTables = () => {
  accountCodes = db.table<IAccountCode, string>(Table.AccountCode);
  accountStorages = db.table<IAccountStorage, string>(Table.AccountStorage);
  storageMapEntries = db.table<IStorageMapEntry, string>(
    Table.StorageMapEntries
  );
  accountAssets = db.table<IAccountAsset, string>(Table.AccountAssets);
  accountAuths = db.table<IAccountAuth, string>(Table.AccountAuth);
  accounts = db.table<IAccount, string>(Table.Accounts);
  addresses = db.table<IAddress, string>(Table.Addresses);
  transactions = db.table<ITransaction, string>(Table.Transactions);
  transactionScripts = db.table<ITransactionScript, string>(
    Table.TransactionScripts
  );
  inputNotes = db.table<IInputNote, string>(Table.InputNotes);
  outputNotes = db.table<IOutputNote, string>(Table.OutputNotes);
  notesScripts = db.table<INotesScript, string>(Table.NotesScripts);
  stateSync = db.table<IStateSync, number>(Table.StateSync);
  blockHeaders = db.table<IBlockHeader, string>(Table.BlockHeaders);
  partialBlockchainNodes = db.table<IPartialBlockchainNode, string>(
    Table.PartialBlockchainNodes
  );
  tags = db.table<ITag, number>(Table.Tags);
  foreignAccountCode = db.table<IForeignAccountCode, string>(
    Table.ForeignAccountCode
  );
  settings = db.table<ISetting, string>(Table.Settings);
  trackedAccounts = db.table<ITrackedAccount, string>(Table.TrackedAccounts);
};

configureDatabase(db);
bindTables();
let dexieInitialized = !runningInNode;

const reinitializeDbForNode = () => {
  if (dexieInitialized) {
    return;
  }
  db.close();
  db = new Dexie(DATABASE_NAME);
  configureDatabase(db);
  bindTables();
  dexieInitialized = true;
};

function indexes(...items: string[]): string {
  return items.join(",");
}

db.on("populate", () => {
  // Populate the stateSync table with default values
  stateSync
    .put({ id: 1, blockNum: "0" } as IStateSync)
    .catch((err: unknown) => logWebStoreError(err, "Failed to populate DB"));
});

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
  nodeIndexedDbBasePath,
};
