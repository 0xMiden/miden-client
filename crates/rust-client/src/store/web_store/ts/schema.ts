import Dexie from "dexie";
import { logWebStoreError } from "./utils.js";

const DATABASE_NAME = "MidenClientDB";

export async function openDatabase(): Promise<boolean> {
  console.log("Opening database...");
  try {
    await db.open();
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
  AccountVaults = "accountVaults",
  AccountAuth = "accountAuth",
  Accounts = "accounts",
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
}

export interface IAccountCode {
  root: string;
  code: Uint8Array;
}

export interface IAccountStorage {
  root: string;
  slots: Blob;
}

export interface IAccountVault {
  root: string;
  assets: Blob;
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

export interface ITransaction {
  id: string;
  details: Blob;
  blockNum: number;
  scriptRoot?: string;
  statusVariant: number;
  status: Blob;
}

export interface ITransactionScript {
  scriptRoot: string;
  txScript?: Blob;
}

export interface IInputNote {
  noteId: string;
  stateDiscriminant: number;
  assets: Blob;
  serialNumber: Blob;
  inputs: Blob;
  scriptRoot: string;
  nullifier: string;
  serializedCreatedAt: string;
  state: Blob;
}

export interface IOutputNote {
  noteId: string;
  recipientDigest: string;
  assets: Blob;
  metadata: Blob;
  stateDiscriminant: number;
  nullifier?: string;
  expectedHeight: number;
  state: Blob;
}

export interface INotesScript {
  scriptRoot: string;
  serializedNoteScript: Blob;
}

export interface IStateSync {
  id: number;
  blockNum: string;
}

export interface IBlockHeader {
  blockNum: string;
  header: Blob;
  partialBlockchainPeaks: Blob;
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

const db = new Dexie(DATABASE_NAME) as Dexie & {
  accountCodes: Dexie.Table<IAccountCode, string>;
  accountStorages: Dexie.Table<IAccountStorage, string>;
  accountVaults: Dexie.Table<IAccountVault, string>;
  accountAuths: Dexie.Table<IAccountAuth, string>;
  accounts: Dexie.Table<IAccount, string>;
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
};

db.version(1).stores({
  [Table.AccountCode]: indexes("root"),
  [Table.AccountStorage]: indexes("root"),
  [Table.AccountVaults]: indexes("root"),
  [Table.AccountAuth]: indexes("pubKey"),
  [Table.Accounts]: indexes(
    "&accountCommitment",
    "id",
    "codeRoot",
    "storageRoot",
    "vaultRoot"
  ),
  [Table.Transactions]: indexes("id"),
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
const accountVaults = db.table<IAccountVault, string>(Table.AccountVaults);
const accountAuths = db.table<IAccountAuth, string>(Table.AccountAuth);
const accounts = db.table<IAccount, string>(Table.Accounts);
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

export {
  db,
  accountCodes,
  accountStorages,
  accountVaults,
  accountAuths,
  accounts,
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
};
