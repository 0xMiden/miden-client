import {
  Adapter,
  AllowedPrivateData,
  MessageSignerWalletAdapterProps,
  MidenTransaction,
  PrivateDataPermission,
  SignKind,
  WalletAdapterNetwork,
  WalletError,
  WalletName,
  WalletNotConnectedError,
  WalletNotReadyError,
  WalletNotSelectedError,
  WalletReadyState,
} from '@miden-sdk/miden-wallet-adapter-base';
import type { FC, ReactNode } from 'react';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useLocalStorage } from './useLocalStorage';
import type { Wallet } from './useWallet';
import { WalletContext } from './useWallet';
import type { NoteFilterTypes } from '@miden-sdk/miden-sdk';

export interface WalletProviderProps {
  children: ReactNode;
  wallets: Adapter[];
  privateDataPermission?: PrivateDataPermission;
  allowedPrivateData?: AllowedPrivateData;
  network?: WalletAdapterNetwork;
  autoConnect?: boolean;
  onError?: (error: WalletError) => void;
  localStorageKey?: string;
}

const initialState: {
  wallet: Wallet | null;
  adapter: Adapter | null;
  address: string | null;
  publicKey: Uint8Array | null;
  connected: boolean;
} = {
  wallet: null,
  adapter: null,
  address: null,
  publicKey: null,
  connected: false,
};

export const WalletProvider: FC<WalletProviderProps> = ({
  children,
  wallets: adapters,
  autoConnect = false,
  privateDataPermission = PrivateDataPermission.UponRequest,
  network = WalletAdapterNetwork.Testnet,
  onError,
  localStorageKey = 'walletName',
  allowedPrivateData = AllowedPrivateData.None,
}) => {
  const [name, setName] = useLocalStorage<WalletName | null>(
    localStorageKey,
    null
  );
  const [{ wallet, adapter, address, publicKey, connected }, setState] =
    useState(initialState);
  const readyState = adapter?.readyState || WalletReadyState.Unsupported;
  const [connecting, setConnecting] = useState(false);
  const [disconnecting, setDisconnecting] = useState(false);
  const isConnecting = useRef(false);
  const isDisconnecting = useRef(false);
  const isUnloading = useRef(false);

  // Wrap adapters to conform to the `Wallet` interface
  const [wallets, setWallets] = useState(() =>
    adapters.map((adapter) => ({
      adapter,
      readyState: adapter.readyState,
    }))
  );

  // When the adapters change, start to listen for changes to their `readyState`
  useEffect(() => {
    // When the adapters change, wrap them to conform to the `Wallet` interface
    setWallets((wallets) =>
      adapters.map((adapter, index) => {
        const wallet = wallets[index];
        // If the wallet hasn't changed, return the same instance
        return wallet &&
          wallet.adapter === adapter &&
          wallet.readyState === adapter.readyState
          ? wallet
          : {
              adapter: adapter,
              readyState: adapter.readyState,
            };
      })
    );

    function handleReadyStateChange(
      this: Adapter,
      readyState: WalletReadyState
    ) {
      setWallets((prevWallets) => {
        const index = prevWallets.findIndex(({ adapter }) => adapter === this);
        if (index === -1) return prevWallets;

        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        const { adapter } = prevWallets[index]!;
        return [
          ...prevWallets.slice(0, index),
          { adapter, readyState },
          ...prevWallets.slice(index + 1),
        ];
      });
    }

    adapters.forEach((adapter) =>
      adapter.on('readyStateChange', handleReadyStateChange, adapter)
    );
    return () =>
      adapters.forEach((adapter) =>
        adapter.off('readyStateChange', handleReadyStateChange, adapter)
      );
  }, [adapters]);

  // When the selected wallet changes, initialize the state
  useEffect(() => {
    const wallet = name && wallets.find(({ adapter }) => adapter.name === name);
    if (wallet) {
      setState({
        wallet,
        adapter: wallet.adapter,
        connected: wallet.adapter.connected,
        address: wallet.adapter.address,
        publicKey: wallet.adapter.publicKey,
      });
    } else {
      setState(initialState);
    }
  }, [name, wallets]);

  // If the window is closing or reloading, ignore disconnect and error events from the adapter
  useEffect(() => {
    function listener() {
      isUnloading.current = true;
    }

    window.addEventListener('beforeunload', listener);
    return () => window.removeEventListener('beforeunload', listener);
  }, [isUnloading]);

  // Handle the adapter's connect event
  const handleConnect = useCallback(() => {
    if (!adapter) return;
    setState((state) => ({
      ...state,
      connected: adapter.connected,
      address: adapter.address,
      publicKey: adapter.publicKey,
    }));
  }, [adapter]);

  // Handle the adapter's disconnect event
  const handleDisconnect = useCallback(() => {
    // Clear the selected wallet unless the window is unloading
    if (!isUnloading.current) setName(null);
  }, [isUnloading, setName]);

  // Handle the adapter's error event, and local errors
  const handleError = useCallback(
    (error: WalletError) => {
      // Call onError unless the window is unloading
      if (!isUnloading.current) (onError || console.error)(error);
      return error;
    },
    [isUnloading, onError]
  );

  // Setup and teardown event listeners when the adapter changes
  useEffect(() => {
    if (adapter) {
      adapter.on('connect', handleConnect);
      adapter.on('disconnect', handleDisconnect);
      adapter.on('error', handleError);
      return () => {
        adapter.off('connect', handleConnect);
        adapter.off('disconnect', handleDisconnect);
        adapter.off('error', handleError);
      };
    }
  }, [adapter, handleConnect, handleDisconnect, handleError]);

  // When the adapter changes, disconnect the old one
  useEffect(() => {
    return () => {
      adapter?.disconnect();
    };
  }, [adapter]);

  // If autoConnect is enabled, try to connect when the adapter changes and is ready
  useEffect(() => {
    if (
      isConnecting.current ||
      connected ||
      !autoConnect ||
      !adapter ||
      !(
        readyState === WalletReadyState.Installed ||
        readyState === WalletReadyState.Loadable
      )
    )
      return;

    (async function () {
      isConnecting.current = true;
      setConnecting(true);
      try {
        await adapter.connect(
          privateDataPermission,
          network,
          allowedPrivateData
        );
      } catch (error: any) {
        // Clear the selected wallet
        setName(null);
        // Don't throw error, but handleError will still be called
      } finally {
        setConnecting(false);
        isConnecting.current = false;
      }
    })();
  }, [isConnecting, connected, autoConnect, adapter, readyState, setName]);

  // Connect the adapter to the wallet
  const connect = useCallback(async () => {
    if (isConnecting.current || isDisconnecting.current || connected) return;
    if (!adapter) throw handleError(new WalletNotSelectedError());

    if (
      !(
        readyState === WalletReadyState.Installed ||
        readyState === WalletReadyState.Loadable
      )
    ) {
      // Clear the selected wallet
      setName(null);

      if (typeof window !== 'undefined') {
        window.open(adapter.url, '_blank');
      }

      throw handleError(new WalletNotReadyError());
    }

    isConnecting.current = true;
    setConnecting(true);
    try {
      await adapter.connect(privateDataPermission, network, allowedPrivateData);
    } catch (error: any) {
      // Clear the selected wallet
      setName(null);
      // Rethrow the error, and handleError will also be called
      throw error;
    } finally {
      setConnecting(false);
      isConnecting.current = false;
    }
  }, [
    isConnecting,
    isDisconnecting,
    connected,
    adapter,
    readyState,
    handleError,
    setName,
  ]);

  // Disconnect the adapter from the wallet
  const disconnect = useCallback(async () => {
    if (isDisconnecting.current) return;
    if (!adapter) return setName(null);

    isDisconnecting.current = true;
    setDisconnecting(true);
    try {
      await adapter.disconnect();
    } catch (error: any) {
      // Clear the selected wallet
      setName(null);
      // Rethrow the error, and handleError will also be called
      throw error;
    } finally {
      setDisconnecting(false);
      isDisconnecting.current = false;
    }
  }, [isDisconnecting, adapter, setName]);

  // Request transaction
  const requestTransaction:
    | MessageSignerWalletAdapterProps['requestTransaction']
    | undefined = useMemo(
    () =>
      adapter && 'requestTransaction' in adapter
        ? async (transaction: MidenTransaction) => {
            if (!connected) throw handleError(new WalletNotConnectedError());
            return await adapter.requestTransaction(transaction);
          }
        : undefined,
    [adapter, handleError, connected]
  );

  // Request assets
  const requestAssets:
    | MessageSignerWalletAdapterProps['requestAssets']
    | undefined = useMemo(
    () =>
      adapter && 'requestAssets' in adapter
        ? async () => {
            if (!connected) throw handleError(new WalletNotConnectedError());
            return await adapter.requestAssets();
          }
        : undefined,
    [adapter, handleError, connected]
  );

  // Request private notes
  const requestPrivateNotes:
    | MessageSignerWalletAdapterProps['requestPrivateNotes']
    | undefined = useMemo(
    () =>
      adapter && 'requestPrivateNotes' in adapter
        ? async (noteFilterType: NoteFilterTypes, noteIds?: string[]) => {
            if (!connected) throw handleError(new WalletNotConnectedError());
            return await adapter.requestPrivateNotes(noteFilterType, noteIds);
          }
        : undefined,
    [adapter, handleError, connected]
  );

  const signBytes: MessageSignerWalletAdapterProps['signBytes'] | undefined =
    useMemo(
      () =>
        adapter && 'signBytes' in adapter
          ? async (message: Uint8Array, kind: SignKind) => {
              if (!connected) throw handleError(new WalletNotConnectedError());
              return await adapter.signBytes(message, kind);
            }
          : undefined,
      [adapter, handleError, connected]
    );

  const importPrivateNote:
    | MessageSignerWalletAdapterProps['importPrivateNote']
    | undefined = useMemo(
    () =>
      adapter && 'importPrivateNote' in adapter
        ? async (note: Uint8Array) => {
            if (!connected) throw handleError(new WalletNotConnectedError());
            return await adapter.importPrivateNote(note);
          }
        : undefined,
    [adapter, handleError, connected]
  );

  const requestConsumableNotes:
    | MessageSignerWalletAdapterProps['requestConsumableNotes']
    | undefined = useMemo(
    () =>
      adapter && 'requestConsumableNotes' in adapter
        ? async () => {
            if (!connected) throw handleError(new WalletNotConnectedError());
            return await adapter.requestConsumableNotes();
          }
        : undefined,
    [adapter, handleError, connected]
  );

  const waitForTransaction:
    | MessageSignerWalletAdapterProps['waitForTransaction']
    | undefined = useMemo(
    () =>
      adapter && 'waitForTransaction' in adapter
        ? async (txId: string, timeout?: number) => {
            if (!connected) throw handleError(new WalletNotConnectedError());
            return await adapter.waitForTransaction(txId, timeout);
          }
        : undefined,
    [adapter, handleError, connected]
  );

  const requestSend:
    | MessageSignerWalletAdapterProps['requestSend']
    | undefined = useMemo(
    () =>
      adapter && 'requestSend' in adapter
        ? async (transaction) => {
            if (!connected) throw handleError(new WalletNotConnectedError());
            return await adapter.requestSend(transaction);
          }
        : undefined,
    [adapter, handleError, connected]
  );

  const requestConsume:
    | MessageSignerWalletAdapterProps['requestConsume']
    | undefined = useMemo(
    () =>
      adapter && 'requestConsume' in adapter
        ? async (transaction) => {
            if (!connected) throw handleError(new WalletNotConnectedError());
            return await adapter.requestConsume(transaction);
          }
        : undefined,
    [adapter, handleError, connected]
  );

  return (
    <WalletContext.Provider
      value={{
        autoConnect,
        wallets,
        wallet,
        address,
        publicKey,
        connected,
        connecting,
        disconnecting,
        select: setName,
        connect,
        disconnect,
        requestTransaction,
        requestAssets,
        requestPrivateNotes,
        signBytes,
        importPrivateNote,
        requestConsumableNotes,
        waitForTransaction,
        requestSend,
        requestConsume,
      }}
    >
      {children}
    </WalletContext.Provider>
  );
};
