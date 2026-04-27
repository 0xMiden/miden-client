import type { InputNoteState, Note, NoteType } from '@miden-sdk/miden-sdk';

export enum WalletAdapterNetwork {
  Testnet = 'testnet',
  Localnet = 'localnet',
}

export type SupportedTransactionVersions = ReadonlySet<any> | null;

export type TransactionOrVersionedTransaction<
  S extends SupportedTransactionVersions
> = S extends null ? any : any | any;

export enum PrivateDataPermission {
  UponRequest = 'UPON_REQUEST', // The App must ask for permission to access private data every time
  Auto = 'AUTO', // The App can access private data without confirmation
}

export enum AllowedPrivateData {
  None = 0,
  Assets = 1 << 0, // 1
  Notes = 1 << 1, // 2
  Storage = 1 << 2, // 4
  All = (1 << 16) - 1, // 65535 (allows for new permissions without requiring a migration)
}

export type SignKind = 'word' | 'signingInputs';

export interface Asset {
  faucetId: string;
  amount: string;
}

export type InputNoteDetails = {
  noteId: string;
  senderAccountId: string | undefined;
  assets: FungibleAssetDetails[];
  noteType: NoteType | undefined;
  nullifier: string;
  state: InputNoteState;
};

export type FungibleAssetDetails = {
  amount: string;
  faucetId: string;
};

// Developer-facing transaction output interface
// Errors from the wallet are thrown as exceptions
export interface TransactionOutput {
  txHash: string;
  outputNotes: Note[];
}

export interface IFailedTransactionOutput {
  errorMessage: string;
}

// Internal wallet response format - uses serialized note data (string[])
// Converted to Note objects for the public API (TransactionOutput)
export interface WalletTransactionSuccessOutput {
  txHash: string;
  outputNotes: string[];
}

export type WalletTransactionOutput =
  | WalletTransactionSuccessOutput
  | IFailedTransactionOutput;
