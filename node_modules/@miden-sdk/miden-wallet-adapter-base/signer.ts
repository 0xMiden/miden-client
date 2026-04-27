import type { NoteFilterTypes } from '@miden-sdk/miden-sdk';
import type { WalletAdapter, WalletAdapterProps } from './adapter';
import { BaseWalletAdapter } from './adapter';
import {
  MidenConsumeTransaction,
  MidenSendTransaction,
  MidenTransaction,
} from './transaction';
import { Asset, InputNoteDetails, SignKind, TransactionOutput } from './types';

export type Adapter =
  | WalletAdapter
  | SignerWalletAdapter
  | MessageSignerWalletAdapter;

export interface SignerWalletAdapterProps<Name extends string = string>
  extends WalletAdapterProps<Name> {}

export type SignerWalletAdapter<Name extends string = string> =
  WalletAdapter<Name> & SignerWalletAdapterProps<Name>;

export abstract class BaseSignerWalletAdapter<Name extends string = string>
  extends BaseWalletAdapter<Name>
  implements SignerWalletAdapter<Name> {}

export interface MessageSignerWalletAdapterProps<Name extends string = string>
  extends WalletAdapterProps<Name> {
  requestTransaction(transaction: MidenTransaction): Promise<string>;
  requestAssets(): Promise<Asset[]>;
  requestPrivateNotes(
    noteFilterType: NoteFilterTypes,
    noteIds?: string[]
  ): Promise<InputNoteDetails[]>;
  signBytes(data: Uint8Array, kind: SignKind): Promise<Uint8Array>;
  importPrivateNote(note: Uint8Array): Promise<string>;
  requestConsumableNotes(): Promise<InputNoteDetails[]>;
  waitForTransaction(
    txId: string,
    timeout?: number
  ): Promise<TransactionOutput>;
  requestSend(transaction: MidenSendTransaction): Promise<string>;
  requestConsume(transaction: MidenConsumeTransaction): Promise<string>;
}

export type MessageSignerWalletAdapter<Name extends string = string> =
  WalletAdapter<Name> & MessageSignerWalletAdapterProps<Name>;

export abstract class BaseMessageSignerWalletAdapter<
    Name extends string = string
  >
  extends BaseSignerWalletAdapter<Name>
  implements MessageSignerWalletAdapter<Name>
{
  abstract requestSend(transaction: MidenSendTransaction): Promise<string>;
  abstract requestConsume(
    transaction: MidenConsumeTransaction
  ): Promise<string>;
  abstract requestTransaction(transaction: MidenTransaction): Promise<string>;
  abstract requestAssets(): Promise<Asset[]>;
  abstract requestPrivateNotes(
    noteFilterType: NoteFilterTypes,
    noteIds?: string[]
  ): Promise<InputNoteDetails[]>;
  abstract signBytes(data: Uint8Array, kind: SignKind): Promise<Uint8Array>;
  abstract importPrivateNote(note: Uint8Array): Promise<string>;
  abstract requestConsumableNotes(): Promise<InputNoteDetails[]>;
  abstract waitForTransaction(
    txId: string,
    timeout?: number
  ): Promise<TransactionOutput>;
}
