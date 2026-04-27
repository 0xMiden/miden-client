import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
  WalletNotConnectedError,
  WalletTransactionError,
  WalletReadyState,
  WalletNotReadyError,
  WalletConnectionError,
  PrivateDataPermission,
  WalletAdapterNetwork,
} from '@miden-sdk/miden-wallet-adapter-base';

// Mock @miden-sdk/miden-sdk before importing adapter
vi.mock('@miden-sdk/miden-sdk', () => ({
  Note: {
    deserialize: vi.fn((bytes: Uint8Array) => ({ mockNote: true, bytes })),
  },
}));

import { Note } from '@miden-sdk/miden-sdk';
import { MidenWalletAdapter } from '../adapter';

function createMockWallet(overrides: Record<string, any> = {}) {
  return {
    address: '0xmockaddress',
    publicKey: new Uint8Array([1, 2, 3]),
    connect: vi.fn(),
    disconnect: vi.fn(),
    requestSend: vi.fn().mockResolvedValue({ transactionId: 'tx-send-1' }),
    requestConsume: vi
      .fn()
      .mockResolvedValue({ transactionId: 'tx-consume-1' }),
    requestTransaction: vi
      .fn()
      .mockResolvedValue({ transactionId: 'tx-custom-1' }),
    requestAssets: vi
      .fn()
      .mockResolvedValue({ assets: [{ faucetId: 'f1', amount: '100' }] }),
    requestPrivateNotes: vi
      .fn()
      .mockResolvedValue({ privateNotes: [{ noteId: 'n1' }] }),
    signBytes: vi
      .fn()
      .mockResolvedValue({ signature: new Uint8Array([9, 8, 7]) }),
    importPrivateNote: vi.fn().mockResolvedValue({ noteId: 'imported-note-1' }),
    requestConsumableNotes: vi
      .fn()
      .mockResolvedValue({ consumableNotes: [{ noteId: 'cn1' }] }),
    waitForTransaction: vi.fn(),
    ...overrides,
  };
}

// Helper to set up a connected adapter by injecting mock wallet via window
async function createConnectedAdapter() {
  const mockWallet = createMockWallet();
  (window as any).midenWallet = mockWallet;

  const adapter = new MidenWalletAdapter();
  // Force readyState to Installed so connect works
  adapter.readyState = WalletReadyState.Installed;

  await adapter.connect(
    PrivateDataPermission.UponRequest,
    WalletAdapterNetwork.Testnet
  );

  return { adapter, mockWallet };
}

describe('MidenWalletAdapter', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    delete (window as any).midenWallet;
    delete (window as any).miden;
  });

  describe('constructor', () => {
    it('initializes with null address and not connecting', () => {
      const adapter = new MidenWalletAdapter();
      expect(adapter.address).toBeNull();
      expect(adapter.publicKey).toBeNull();
      expect(adapter.connecting).toBe(false);
      expect(adapter.connected).toBe(false);
    });

    it('sets readyState to NotDetected when wallet is not present', () => {
      const adapter = new MidenWalletAdapter();
      expect(adapter.readyState).toBe(WalletReadyState.NotDetected);
    });
  });

  describe('not connected errors', () => {
    const methods: Array<{
      name: string;
      call: (a: MidenWalletAdapter) => Promise<any>;
    }> = [
      {
        name: 'requestSend',
        call: (a) =>
          a.requestSend({
            senderAddress: 's',
            recipientAddress: 'r',
            faucetId: 'f',
            noteType: 'public',
            amount: 1,
          }),
      },
      {
        name: 'requestConsume',
        call: (a) =>
          a.requestConsume({
            faucetId: 'f',
            noteId: 'n',
            noteType: 'public',
            amount: 1,
          }),
      },
      {
        name: 'requestTransaction',
        call: (a) =>
          a.requestTransaction({ type: 'send' as any, payload: {} as any }),
      },
      {
        name: 'requestAssets',
        call: (a) => a.requestAssets(),
      },
      {
        name: 'requestPrivateNotes',
        call: (a) => a.requestPrivateNotes('All' as any),
      },
      {
        name: 'signBytes',
        call: (a) => a.signBytes(new Uint8Array([1]), 'word'),
      },
      {
        name: 'importPrivateNote',
        call: (a) => a.importPrivateNote(new Uint8Array([1])),
      },
      {
        name: 'requestConsumableNotes',
        call: (a) => a.requestConsumableNotes(),
      },
      {
        name: 'waitForTransaction',
        call: (a) => a.waitForTransaction('tx-1'),
      },
    ];

    for (const { name, call } of methods) {
      it(`${name} throws WalletNotConnectedError when not connected`, async () => {
        const adapter = new MidenWalletAdapter();
        const errorHandler = vi.fn();
        adapter.on('error', errorHandler);

        await expect(call(adapter)).rejects.toThrow(WalletNotConnectedError);
        expect(errorHandler).toHaveBeenCalledWith(
          expect.any(WalletNotConnectedError)
        );
      });
    }
  });

  describe('connected operations', () => {
    it('requestSend delegates to wallet and returns transactionId', async () => {
      const { adapter, mockWallet } = await createConnectedAdapter();
      const tx = {
        senderAddress: 's',
        recipientAddress: 'r',
        faucetId: 'f',
        noteType: 'public' as const,
        amount: 100,
      };
      const result = await adapter.requestSend(tx);
      expect(mockWallet.requestSend).toHaveBeenCalledWith(tx);
      expect(result).toBe('tx-send-1');
    });

    it('requestConsume delegates to wallet and returns transactionId', async () => {
      const { adapter, mockWallet } = await createConnectedAdapter();
      const tx = {
        faucetId: 'f',
        noteId: 'n',
        noteType: 'public' as const,
        amount: 50,
      };
      const result = await adapter.requestConsume(tx);
      expect(mockWallet.requestConsume).toHaveBeenCalledWith(tx);
      expect(result).toBe('tx-consume-1');
    });

    it('requestTransaction delegates to wallet and returns transactionId', async () => {
      const { adapter, mockWallet } = await createConnectedAdapter();
      const tx = { type: 'send' as any, payload: {} as any };
      const result = await adapter.requestTransaction(tx);
      expect(mockWallet.requestTransaction).toHaveBeenCalledWith(tx);
      expect(result).toBe('tx-custom-1');
    });

    it('requestAssets returns assets array', async () => {
      const { adapter } = await createConnectedAdapter();
      const assets = await adapter.requestAssets();
      expect(assets).toEqual([{ faucetId: 'f1', amount: '100' }]);
    });

    it('requestPrivateNotes returns notes array', async () => {
      const { adapter } = await createConnectedAdapter();
      const notes = await adapter.requestPrivateNotes('All' as any);
      expect(notes).toEqual([{ noteId: 'n1' }]);
    });

    it('signBytes returns signature', async () => {
      const { adapter } = await createConnectedAdapter();
      const sig = await adapter.signBytes(new Uint8Array([1, 2]), 'word');
      expect(sig).toEqual(new Uint8Array([9, 8, 7]));
    });

    it('importPrivateNote returns noteId', async () => {
      const { adapter } = await createConnectedAdapter();
      const noteId = await adapter.importPrivateNote(new Uint8Array([1]));
      expect(noteId).toBe('imported-note-1');
    });

    it('requestConsumableNotes returns consumable notes', async () => {
      const { adapter } = await createConnectedAdapter();
      const notes = await adapter.requestConsumableNotes();
      expect(notes).toEqual([{ noteId: 'cn1' }]);
    });
  });

  describe('waitForTransaction', () => {
    it('success path: deserializes notes and returns TransactionOutput', async () => {
      // base64 of [1,2,3] = "AQID"
      const { adapter, mockWallet } = await createConnectedAdapter();
      mockWallet.waitForTransaction.mockResolvedValue({
        txHash: 'hash-123',
        outputNotes: ['AQID', 'BAUG'],
      });

      const result = await adapter.waitForTransaction('tx-1');

      expect(result.txHash).toBe('hash-123');
      expect(result.outputNotes).toHaveLength(2);
      expect(Note.deserialize).toHaveBeenCalledTimes(2);
    });

    it('error path: throws WalletTransactionError on errorMessage', async () => {
      const { adapter, mockWallet } = await createConnectedAdapter();
      mockWallet.waitForTransaction.mockResolvedValue({
        errorMessage: 'tx failed',
      });

      const errorHandler = vi.fn();
      adapter.on('error', errorHandler);

      await expect(adapter.waitForTransaction('tx-1')).rejects.toThrow(
        WalletTransactionError
      );
      expect(errorHandler).toHaveBeenCalled();
    });
  });

  describe('connect', () => {
    it('throws WalletNotReadyError if readyState is not Installed', async () => {
      const adapter = new MidenWalletAdapter();
      const errorHandler = vi.fn();
      adapter.on('error', errorHandler);

      await expect(
        adapter.connect(
          PrivateDataPermission.UponRequest,
          WalletAdapterNetwork.Testnet
        )
      ).rejects.toThrow(WalletNotReadyError);
    });

    it('connects successfully and emits connect event', async () => {
      const mockWallet = createMockWallet();
      (window as any).midenWallet = mockWallet;

      const adapter = new MidenWalletAdapter();
      adapter.readyState = WalletReadyState.Installed;

      const connectHandler = vi.fn();
      adapter.on('connect', connectHandler);

      await adapter.connect(
        PrivateDataPermission.UponRequest,
        WalletAdapterNetwork.Testnet
      );

      expect(adapter.address).toBe('0xmockaddress');
      expect(adapter.publicKey).toEqual(new Uint8Array([1, 2, 3]));
      expect(adapter.connected).toBe(true);
      expect(adapter.connecting).toBe(false);
      expect(connectHandler).toHaveBeenCalledWith('0xmockaddress');
    });

    it('throws WalletConnectionError if wallet has no address after connect', async () => {
      const mockWallet = createMockWallet({ address: undefined });
      (window as any).midenWallet = mockWallet;

      const adapter = new MidenWalletAdapter();
      adapter.readyState = WalletReadyState.Installed;

      await expect(
        adapter.connect(
          PrivateDataPermission.UponRequest,
          WalletAdapterNetwork.Testnet
        )
      ).rejects.toThrow(WalletConnectionError);
    });

    it('does nothing if already connected', async () => {
      const { adapter, mockWallet } = await createConnectedAdapter();
      // Call connect again
      await adapter.connect(
        PrivateDataPermission.UponRequest,
        WalletAdapterNetwork.Testnet
      );
      // wallet.connect should have been called only once (during initial connect)
      expect(mockWallet.connect).toHaveBeenCalledTimes(1);
    });
  });

  describe('disconnect', () => {
    it('clears address, publicKey, and emits disconnect', async () => {
      const { adapter } = await createConnectedAdapter();

      const disconnectHandler = vi.fn();
      adapter.on('disconnect', disconnectHandler);

      await adapter.disconnect();

      expect(adapter.address).toBeNull();
      expect(adapter.publicKey).toBeNull();
      expect(adapter.connected).toBe(false);
      expect(disconnectHandler).toHaveBeenCalled();
    });

    it('emits error if wallet.disconnect fails', async () => {
      const { adapter, mockWallet } = await createConnectedAdapter();
      mockWallet.disconnect.mockRejectedValue(new Error('disconnect failed'));

      const errorHandler = vi.fn();
      adapter.on('error', errorHandler);

      await adapter.disconnect();
      expect(errorHandler).toHaveBeenCalled();
    });
  });

  describe('error wrapping', () => {
    it('wraps wallet method failures in WalletTransactionError', async () => {
      const { adapter, mockWallet } = await createConnectedAdapter();
      mockWallet.requestSend.mockRejectedValue(new Error('network error'));

      await expect(
        adapter.requestSend({
          senderAddress: 's',
          recipientAddress: 'r',
          faucetId: 'f',
          noteType: 'public',
          amount: 1,
        })
      ).rejects.toThrow(WalletTransactionError);
    });
  });
});
