import Dexie from "dexie";
export declare function openDatabase(): Promise<boolean>;
export interface IAccountCode {
  root: string;
  code: Blob;
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
  accountSeed: Blob | null;
  accountCommitment: string;
  locked: boolean;
}
export interface ITransaction {
  id: string;
  details: Blob;
  scriptRoot: string;
  blockNum: number;
  commitHeight: number;
  discardCause: Blob | null;
}
export interface ITransactionScript {
  scriptRoot: string;
  script: Blob;
}
export interface IInputNote {
  noteId: string;
  stateDiscriminant: string;
  assets: string;
  serialNumber: Blob;
  inputs: Blob;
  scriptRoot: string;
  nullifier: string;
  createdAt: BigInt;
}
export interface IOutputNote {
  noteId: string;
  recipientDigest: string;
  assets: Blob;
  metadata: Blob;
  stateDiscriminant: string;
  nullifier: string;
  expectedHeight: BigInt;
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
  blockNum: number;
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
  sourceNoteId: string;
  sourceAccountId: string;
}
export interface IForeignAccountCode {
  accountId: string;
  codeRoot: string;
}
export interface ISetting {
  key: string;
  value: Blob;
}
declare const db: Dexie & {
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
  blockHeaders: Dexie.Table<IBlockHeader, number>;
  partialBlockchainNodes: Dexie.Table<IPartialBlockchainNode, string>;
  tags: Dexie.Table<ITag, number>;
  foreignAccountCode: Dexie.Table<IForeignAccountCode, string>;
  settings: Dexie.Table<ISetting, string>;
};
declare const accountCodes: import("dexie").Table<
  IAccountCode,
  string,
  IAccountCode
>;
declare const accountStorages: import("dexie").Table<
  IAccountStorage,
  string,
  IAccountStorage
>;
declare const accountVaults: import("dexie").Table<
  IAccountVault,
  string,
  IAccountVault
>;
declare const accountAuths: import("dexie").Table<
  IAccountAuth,
  string,
  IAccountAuth
>;
declare const accounts: import("dexie").Table<IAccount, string, IAccount>;
declare const transactions: import("dexie").Table<
  ITransaction,
  string,
  ITransaction
>;
declare const transactionScripts: import("dexie").Table<
  ITransactionScript,
  string,
  ITransactionScript
>;
declare const inputNotes: import("dexie").Table<IInputNote, string, IInputNote>;
declare const outputNotes: import("dexie").Table<
  IOutputNote,
  string,
  IOutputNote
>;
declare const notesScripts: import("dexie").Table<
  INotesScript,
  string,
  INotesScript
>;
declare const stateSync: import("dexie").Table<IStateSync, number, IStateSync>;
declare const blockHeaders: import("dexie").Table<
  IBlockHeader,
  number,
  IBlockHeader
>;
declare const partialBlockchainNodes: import("dexie").Table<
  IPartialBlockchainNode,
  string,
  IPartialBlockchainNode
>;
declare const tags: import("dexie").Table<ITag, number, ITag>;
declare const foreignAccountCode: import("dexie").Table<
  IForeignAccountCode,
  string,
  IForeignAccountCode
>;
declare const settings: import("dexie").Table<ISetting, string, ISetting>;
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
  settings,
};
