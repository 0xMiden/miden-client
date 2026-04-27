import type { FC, MouseEventHandler } from 'react';
import { useCallback, useMemo } from 'react';
import { useWallet } from '@miden-sdk/miden-wallet-adapter-react';
import type { ButtonProps } from './Button';
import { Button } from './Button';
import { WalletIcon } from './WalletIcon';
import { AllowedPrivateData, PrivateDataPermission, WalletAdapterNetwork } from '@miden-sdk/miden-wallet-adapter-base';

export const WalletConnectButton: FC<ButtonProps> = ({
  children,
  disabled,
  onClick,
  privateDataPermission,
  network,
  allowedPrivateData,
  ...props
}) => {
  const { wallet, connect, connecting, connected } = useWallet();

  const handleClick: MouseEventHandler<HTMLButtonElement> = useCallback(
    (event) => {
      if (onClick) onClick(event);
      // eslint-disable-next-line @typescript-eslint/no-empty-function
      if (!event.defaultPrevented)
        connect(
          privateDataPermission || PrivateDataPermission.UponRequest,
          network || WalletAdapterNetwork.Testnet,
          allowedPrivateData ?? AllowedPrivateData.None
        ).catch(() => {});
    },
    [onClick, connect]
  );

  const content = useMemo(() => {
    if (children) return children;
    if (connecting) return 'Connecting ...';
    if (connected) return 'Connected';
    if (wallet) return 'Connect';
    return 'Connect Wallet';
  }, [children, connecting, connected, wallet]);

  return (
    <Button
      className="wallet-adapter-button-trigger"
      disabled={disabled || !wallet || connecting || connected}
      startIcon={wallet ? <WalletIcon wallet={wallet} /> : undefined}
      onClick={handleClick}
      {...props}
    >
      {content}
    </Button>
  );
};
