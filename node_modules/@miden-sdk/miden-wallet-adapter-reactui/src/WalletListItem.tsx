import type { FC, MouseEventHandler } from 'react';
import { WalletReadyState } from '@miden-sdk/miden-wallet-adapter-base';
import { Wallet } from '@miden-sdk/miden-wallet-adapter-react';
import { Button } from './Button';
import { WalletIcon } from './WalletIcon';

export interface WalletListItemProps {
  handleClick: MouseEventHandler<HTMLButtonElement>;
  tabIndex?: number;
  wallet: Wallet;
}

export const WalletListItem: FC<WalletListItemProps> = ({
  handleClick,
  tabIndex,
  wallet,
}) => {
  return (
    <li>
      <Button
        onClick={handleClick}
        startIcon={<WalletIcon wallet={wallet} />}
        tabIndex={tabIndex}
      >
        {wallet.adapter.name}
        {wallet.readyState === WalletReadyState.Installed && (
          <>
            <span>Installed</span>
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="25"
              height="24"
              viewBox="0 0 25 24"
              fill="none"
            >
              <path
                d="M10.1003 5L8.5 6.5745L13.6981 11.7L8.5 16.8255L10.1003 18.4L16.91 11.7L10.1003 5Z"
                fill="black"
              />
            </svg>
          </>
        )}
      </Button>
    </li>
  );
};
