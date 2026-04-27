// Legacy exports for backward compatibility
export * from './WalletProvider';
export * from './useLocalStorage';
export * from './useWallet';

// MidenFi Signer Provider - unified provider for @miden-sdk/react integration
export {
  MidenFiSignerProvider,
  useMidenFiWallet,
  WalletContext,
  type MidenFiSignerProviderProps,
  type Wallet,
  type WalletContextState,
  type MidenFiWalletContextState,
} from './MidenFiSignerProvider';