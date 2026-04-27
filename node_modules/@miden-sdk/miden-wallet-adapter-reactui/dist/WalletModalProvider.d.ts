import type { FC, ReactNode } from 'react';
import type { WalletModalProps } from './WalletModal';
export interface WalletModalProviderProps extends WalletModalProps {
    children: ReactNode;
}
export declare const WalletModalProvider: FC<WalletModalProviderProps>;
