import type { TransactionRequest } from '@miden-sdk/miden-sdk';
import { u8ToB64 } from './helpers';

export type NoteTypeString = 'public' | 'private';

export interface MidenSendTransaction {
  senderAddress: string;
  recipientAddress: string;
  faucetId: string;
  noteType: NoteTypeString;
  amount: number;
  recallBlocks?: number;
}

export class SendTransaction implements MidenSendTransaction {
  senderAddress: string;
  recipientAddress: string;
  faucetId: string;
  noteType: NoteTypeString;
  amount: number;
  recallBlocks?: number;

  constructor(
    sender: string,
    recipient: string,
    faucetId: string,
    noteType: NoteTypeString,
    amount: number,
    recallBlocks?: number
  ) {
    this.senderAddress = sender;
    this.recipientAddress = recipient;
    this.faucetId = faucetId;
    this.noteType = noteType;
    this.amount = amount;
    this.recallBlocks = recallBlocks;
  }
}

export interface MidenConsumeTransaction {
  faucetId: string;
  noteId: string;
  noteType: NoteTypeString;
  amount: number;
  noteBytes?: string;
}

export class ConsumeTransaction implements MidenConsumeTransaction {
  faucetId: string;
  noteId: string;
  noteType: NoteTypeString;
  amount: number;
  noteBytes?: string;

  constructor(
    faucetId: string,
    noteId: string,
    noteType: NoteTypeString,
    amount: number,
    noteBytes?: Uint8Array
  ) {
    this.faucetId = faucetId;
    this.noteId = noteId;
    this.noteType = noteType;
    this.amount = amount;
    this.noteBytes = noteBytes ? u8ToB64(noteBytes) : undefined;
  }
}

export interface MidenCustomTransaction {
  address: string;
  recipientAddress: string;
  transactionRequest: string;
  inputNoteIds?: string[];
  importNotes?: string[];
}

export class CustomTransaction implements MidenCustomTransaction {
  address: string;
  recipientAddress: string;
  transactionRequest: string;
  inputNoteIds?: string[];
  importNotes?: string[];

  constructor(
    address: string,
    recipientAddress: string,
    transactionRequest: TransactionRequest,
    inputNotesIds?: string[],
    inputNoteBytes?: Uint8Array[]
  ) {
    this.address = address;
    this.recipientAddress = recipientAddress;
    const requestBytes = transactionRequest.serialize();
    const base64 = u8ToB64(requestBytes);
    this.transactionRequest = base64;
    this.inputNoteIds = inputNotesIds;
    this.importNotes = inputNoteBytes?.map((note) => u8ToB64(note));
  }
}

export enum TransactionType {
  Send = 'send',
  Consume = 'consume',
  Custom = 'custom',
}
export type TransactionPayload =
  | MidenSendTransaction
  | MidenConsumeTransaction
  | MidenCustomTransaction;

export interface MidenTransaction {
  type: TransactionType;
  payload: TransactionPayload;
}

export class Transaction implements MidenTransaction {
  type: TransactionType;
  payload: TransactionPayload;

  constructor(type: TransactionType, payload: TransactionPayload) {
    this.type = type;
    this.payload = payload;
  }

  static createSendTransaction(
    sender: string,
    recipient: string,
    faucetId: string,
    noteType: NoteTypeString,
    amount: number,
    recallBlocks?: number
  ) {
    const sendTransaction = new SendTransaction(
      sender,
      recipient,
      faucetId,
      noteType,
      amount,
      recallBlocks
    );
    return new Transaction(TransactionType.Send, sendTransaction);
  }

  static createConsumeTransaction(
    faucetId: string,
    noteId: string,
    noteType: NoteTypeString,
    amount: number,
    noteBytes?: Uint8Array
  ) {
    const consumeTransaction = new ConsumeTransaction(
      faucetId,
      noteId,
      noteType,
      amount,
      noteBytes
    );
    return new Transaction(TransactionType.Consume, consumeTransaction);
  }

  static createCustomTransaction(
    address: string,
    recipientAddress: string,
    transactionRequest: TransactionRequest,
    inputNoteIds?: string[],
    noteBytes?: Uint8Array[]
  ) {
    const transactionBytes = new CustomTransaction(
      address,
      recipientAddress,
      transactionRequest,
      inputNoteIds,
      noteBytes
    );
    return new Transaction(TransactionType.Custom, transactionBytes);
  }
}
