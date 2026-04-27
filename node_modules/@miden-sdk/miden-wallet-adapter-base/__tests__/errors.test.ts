import { describe, it, expect } from 'vitest';
import {
  WalletError,
  WalletNotReadyError,
  WalletLoadError,
  WalletConfigError,
  WalletConnectionError,
  WalletNotSelectedError,
  WalletDisconnectedError,
  WalletDisconnectionError,
  WalletAccountError,
  WalletAddressError,
  WalletKeypairError,
  WalletNotConnectedError,
  WalletSendTransactionError,
  WalletSignMessageError,
  WalletSignTransactionError,
  WalletTimeoutError,
  WalletWindowBlockedError,
  WalletWindowClosedError,
  WalletDecryptionNotAllowedError,
  WalletPrivateDataPermissionError,
  WalletDecryptionError,
  WalletRecordsError,
  WalletTransactionError,
} from '../errors';

const errorClasses = [
  { Class: WalletNotReadyError, name: 'WalletNotReadyError' },
  { Class: WalletLoadError, name: 'WalletLoadError' },
  { Class: WalletConfigError, name: 'WalletConfigError' },
  { Class: WalletConnectionError, name: 'WalletConnectionError' },
  { Class: WalletNotSelectedError, name: 'WalletNotSelectedError' },
  { Class: WalletDisconnectedError, name: 'WalletDisconnectedError' },
  { Class: WalletDisconnectionError, name: 'WalletDisconnectionError' },
  { Class: WalletAccountError, name: 'WalletAccountError' },
  { Class: WalletAddressError, name: 'WalletAddressError' },
  { Class: WalletKeypairError, name: 'WalletKeypairError' },
  { Class: WalletNotConnectedError, name: 'WalletNotConnectedError' },
  { Class: WalletSendTransactionError, name: 'WalletSendTransactionError' },
  { Class: WalletSignMessageError, name: 'WalletSignMessageError' },
  { Class: WalletSignTransactionError, name: 'WalletSignTransactionError' },
  { Class: WalletTimeoutError, name: 'WalletTimeoutError' },
  { Class: WalletWindowBlockedError, name: 'WalletWindowBlockedError' },
  { Class: WalletWindowClosedError, name: 'WalletWindowClosedError' },
  {
    Class: WalletDecryptionNotAllowedError,
    name: 'WalletDecryptionNotAllowedError',
  },
  {
    Class: WalletPrivateDataPermissionError,
    name: 'WalletPrivateDataPermissionError',
  },
  { Class: WalletDecryptionError, name: 'WalletDecryptionError' },
  { Class: WalletRecordsError, name: 'WalletRecordsError' },
  { Class: WalletTransactionError, name: 'WalletTransactionError' },
] as const;

describe('WalletError', () => {
  it('extends Error', () => {
    const err = new WalletError('test message');
    expect(err).toBeInstanceOf(Error);
    expect(err).toBeInstanceOf(WalletError);
  });

  it('sets message property', () => {
    const err = new WalletError('my message');
    expect(err.message).toBe('my message');
  });

  it('stores inner error', () => {
    const inner = new Error('inner');
    const err = new WalletError('outer', inner);
    expect(err.error).toBe(inner);
  });
});

describe('error subclasses', () => {
  for (const { Class, name } of errorClasses) {
    describe(name, () => {
      it('extends WalletError and Error', () => {
        const err = new Class();
        expect(err).toBeInstanceOf(WalletError);
        expect(err).toBeInstanceOf(Error);
      });

      it('has correct name property', () => {
        const err = new Class();
        expect(err.name).toBe(name);
      });

      it('stores inner error', () => {
        const inner = new Error('cause');
        const err = new Class('msg', inner);
        expect(err.error).toBe(inner);
        expect(err.message).toBe('msg');
      });
    });
  }
});
