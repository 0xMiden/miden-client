import type { FC } from 'react';
import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import { useWallet } from '@miden-sdk/miden-wallet-adapter-react';
import type { ButtonProps } from './Button';
import { Button } from './Button';
import { useWalletModal } from './useWalletModal';
import { WalletConnectButton } from './WalletConnectButton';
import { WalletIcon } from './WalletIcon';
import { WalletModalButton } from './WalletModalButton';

export const WalletMultiButton: FC<ButtonProps> = ({ children, ...props }) => {
  const { address, wallet, disconnect } = useWallet();
  const { setVisible } = useWalletModal();
  const [copied, setCopied] = useState(false);
  const [active, setActive] = useState(false);
  const ref = useRef<HTMLUListElement>(null);

  const content = useMemo(() => {
    if (children) return children;
    if (!wallet || !address) return null;

    const underscoreIndex = address.indexOf('_');
    const frontPart = address.slice(0, 6);
    const middlePart = address.slice(underscoreIndex - 4, underscoreIndex);
    return `${frontPart}...${middlePart}`;
  }, [children, wallet, address]);

  const copyAddress = useCallback(async () => {
    if (address) {
      await navigator.clipboard.writeText(address);
      setCopied(true);
      setTimeout(() => setCopied(false), 400);
    }
  }, [address]);

  const openDropdown = useCallback(() => {
    setActive(true);
  }, []);

  const closeDropdown = useCallback(() => {
    setActive(false);
  }, []);

  const openModal = useCallback(() => {
    setVisible(true);
    closeDropdown();
  }, [setVisible, closeDropdown]);

  useEffect(() => {
    const listener = (event: MouseEvent | TouchEvent) => {
      const node = ref.current;

      // Do nothing if clicking dropdown or its descendants
      if (!node || node.contains(event.target as Node)) return;

      closeDropdown();
    };

    document.addEventListener('mousedown', listener);
    document.addEventListener('touchstart', listener);

    return () => {
      document.removeEventListener('mousedown', listener);
      document.removeEventListener('touchstart', listener);
    };
  }, [ref, closeDropdown]);

  if (!wallet)
    return <WalletModalButton {...props}>{children}</WalletModalButton>;
  if (!address)
    return <WalletConnectButton {...props}>{children}</WalletConnectButton>;

  return (
    <div className="wallet-adapter-dropdown">
      <Button
        aria-expanded={active}
        className="wallet-adapter-button-trigger"
        style={{
          pointerEvents: active ? 'none' : 'auto',
          width: '100%',
          backgroundColor: '#FFFFFF',
          border: '1px solid #D7D7D7',
          color: 'black',
          ...props.style
        }}
        onClick={openDropdown}
        startIcon={<WalletIcon wallet={wallet} />}
        {...props}
      >
        {content}
      </Button>
      <ul
        aria-label="dropdown-list"
        className={`wallet-adapter-dropdown-list ${
          active && 'wallet-adapter-dropdown-list-active'
        }`}
        ref={ref}
        role="menu"
      >
        <li
          onClick={copyAddress}
          className="wallet-adapter-dropdown-list-item"
          role="menuitem"
        >
          {copied ? 'Copied' : 'Copy address'}
        </li>
        <li
          onClick={openModal}
          className="wallet-adapter-dropdown-list-item"
          role="menuitem"
        >
          Change wallet
        </li>
        <li
          onClick={disconnect}
          className="wallet-adapter-dropdown-list-item"
          role="menuitem"
        >
          Disconnect
        </li>
      </ul>
    </div>
  );
};
