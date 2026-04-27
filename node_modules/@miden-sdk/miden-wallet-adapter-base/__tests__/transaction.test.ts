import { describe, it, expect, vi } from 'vitest';
import {
  SendTransaction,
  ConsumeTransaction,
  CustomTransaction,
  Transaction,
  TransactionType,
} from '../transaction';
import { u8ToB64 } from '../helpers';

describe('TransactionType enum', () => {
  it('has correct values', () => {
    expect(TransactionType.Send).toBe('send');
    expect(TransactionType.Consume).toBe('consume');
    expect(TransactionType.Custom).toBe('custom');
  });
});

describe('SendTransaction', () => {
  it('sets all fields correctly', () => {
    const tx = new SendTransaction(
      'sender-addr',
      'recipient-addr',
      'faucet-123',
      'public',
      1000
    );
    expect(tx.senderAddress).toBe('sender-addr');
    expect(tx.recipientAddress).toBe('recipient-addr');
    expect(tx.faucetId).toBe('faucet-123');
    expect(tx.noteType).toBe('public');
    expect(tx.amount).toBe(1000);
    expect(tx.recallBlocks).toBeUndefined();
  });

  it('sets optional recallBlocks', () => {
    const tx = new SendTransaction(
      'sender',
      'recipient',
      'faucet',
      'private',
      500,
      10
    );
    expect(tx.recallBlocks).toBe(10);
  });
});

describe('ConsumeTransaction', () => {
  it('sets fields correctly', () => {
    const tx = new ConsumeTransaction('faucet-1', 'note-1', 'public', 100);
    expect(tx.faucetId).toBe('faucet-1');
    expect(tx.noteId).toBe('note-1');
    expect(tx.noteType).toBe('public');
    expect(tx.amount).toBe(100);
    expect(tx.noteBytes).toBeUndefined();
  });

  it('converts noteBytes Uint8Array to base64', () => {
    const bytes = new Uint8Array([1, 2, 3]);
    const tx = new ConsumeTransaction('faucet', 'note', 'private', 50, bytes);
    expect(tx.noteBytes).toBe(u8ToB64(bytes));
  });

  it('leaves noteBytes undefined when not provided', () => {
    const tx = new ConsumeTransaction('faucet', 'note', 'public', 50);
    expect(tx.noteBytes).toBeUndefined();
  });
});

describe('CustomTransaction', () => {
  it('serializes transactionRequest to base64', () => {
    const mockBytes = new Uint8Array([10, 20, 30]);
    const mockRequest = {
      serialize: vi.fn().mockReturnValue(mockBytes),
    } as any;

    const tx = new CustomTransaction('addr', 'recipient', mockRequest);
    expect(mockRequest.serialize).toHaveBeenCalled();
    expect(tx.transactionRequest).toBe(u8ToB64(mockBytes));
    expect(tx.address).toBe('addr');
    expect(tx.recipientAddress).toBe('recipient');
  });

  it('sets optional inputNoteIds', () => {
    const mockRequest = {
      serialize: vi.fn().mockReturnValue(new Uint8Array([1])),
    } as any;
    const tx = new CustomTransaction('addr', 'recipient', mockRequest, [
      'id1',
      'id2',
    ]);
    expect(tx.inputNoteIds).toEqual(['id1', 'id2']);
  });

  it('converts importNotes from Uint8Array[] to base64 strings', () => {
    const mockRequest = {
      serialize: vi.fn().mockReturnValue(new Uint8Array([1])),
    } as any;
    const noteBytes = [new Uint8Array([4, 5]), new Uint8Array([6, 7])];
    const tx = new CustomTransaction(
      'addr',
      'recipient',
      mockRequest,
      ['id1'],
      noteBytes
    );
    expect(tx.importNotes).toEqual([
      u8ToB64(noteBytes[0]),
      u8ToB64(noteBytes[1]),
    ]);
  });

  it('leaves optional fields undefined when not provided', () => {
    const mockRequest = {
      serialize: vi.fn().mockReturnValue(new Uint8Array([1])),
    } as any;
    const tx = new CustomTransaction('addr', 'recipient', mockRequest);
    expect(tx.inputNoteIds).toBeUndefined();
    expect(tx.importNotes).toBeUndefined();
  });
});

describe('Transaction factory', () => {
  it('createSendTransaction returns correct type and payload', () => {
    const tx = Transaction.createSendTransaction(
      'sender',
      'recipient',
      'faucet',
      'public',
      100
    );
    expect(tx.type).toBe(TransactionType.Send);
    expect(tx.payload).toBeInstanceOf(SendTransaction);
    const payload = tx.payload as SendTransaction;
    expect(payload.senderAddress).toBe('sender');
    expect(payload.amount).toBe(100);
  });

  it('createConsumeTransaction returns correct type and payload', () => {
    const tx = Transaction.createConsumeTransaction(
      'faucet',
      'note-id',
      'private',
      200
    );
    expect(tx.type).toBe(TransactionType.Consume);
    expect(tx.payload).toBeInstanceOf(ConsumeTransaction);
    const payload = tx.payload as ConsumeTransaction;
    expect(payload.faucetId).toBe('faucet');
    expect(payload.noteId).toBe('note-id');
  });

  it('createCustomTransaction returns correct type and payload', () => {
    const mockRequest = {
      serialize: vi.fn().mockReturnValue(new Uint8Array([1, 2])),
    } as any;
    const tx = Transaction.createCustomTransaction(
      'addr',
      'recipient',
      mockRequest
    );
    expect(tx.type).toBe(TransactionType.Custom);
    expect(tx.payload).toBeInstanceOf(CustomTransaction);
  });
});
