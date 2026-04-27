import type { FC, MouseEvent } from 'react';
import {
  useCallback,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import { createPortal } from 'react-dom';
import {
  AllowedPrivateData,
  PrivateDataPermission,
  WalletAdapterNetwork,
  WalletName,
  WalletReadyState,
} from '@miden-sdk/miden-wallet-adapter-base';
import { useWallet, Wallet } from '@miden-sdk/miden-wallet-adapter-react';
import { useWalletModal } from './useWalletModal';
import { WalletListItem } from './WalletListItem';
import { DiscoverMidenMessage } from './DiscoverMidenMessage';

export interface WalletModalProps {
  className?: string;
  container?: string;
  privateDataPermission?: PrivateDataPermission;
  network?: WalletAdapterNetwork;
  allowedPrivateData?: AllowedPrivateData;
}

export const WalletModal: FC<WalletModalProps> = ({
  className = '',
  container = 'body',
  privateDataPermission,
  network,
  allowedPrivateData,
}) => {
  const ref = useRef<HTMLDivElement>(null);
  const { wallets, select, connect, wallet } = useWallet();
  const { setVisible } = useWalletModal();
  const [fadeIn, setFadeIn] = useState(false);
  const [portal, setPortal] = useState<Element | null>(null);

  const [installedWallets, otherWallets] = useMemo(() => {
    const installed: Wallet[] = [];
    const notDetected: Wallet[] = [];
    const loadable: Wallet[] = [];

    for (const wallet of wallets) {
      if (wallet.readyState === WalletReadyState.NotDetected) {
        notDetected.push(wallet);
      } else if (wallet.readyState === WalletReadyState.Loadable) {
        loadable.push(wallet);
      } else if (wallet.readyState === WalletReadyState.Installed) {
        installed.push(wallet);
      }
    }

    return [installed, [...loadable, ...notDetected]];
  }, [wallets]);

  const getStartedWallet = useMemo(() => {
    return installedWallets.length
      ? // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        installedWallets[0]!
      : wallets.find(
          (wallet: { adapter: { name: WalletName } }) =>
            wallet.adapter.name === 'Miden Wallet'
        ) ||
          // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
          otherWallets[0]!;
  }, [installedWallets, wallets, otherWallets]);

  const otherInstalledWallets = useMemo(() => {
    return installedWallets.filter(
      (wallet) => wallet.adapter.name !== getStartedWallet.adapter.name
    );
  }, [installedWallets, getStartedWallet]);

  const hideModal = useCallback(() => {
    setFadeIn(false);
    setTimeout(() => setVisible(false), 150);
  }, [setVisible]);

  const handleClose = useCallback(
    (event: MouseEvent) => {
      event.preventDefault();
      hideModal();
    },
    [hideModal]
  );

  const handleWalletClick = useCallback(
    (event: MouseEvent, walletName: WalletName) => {
      select(walletName);
      handleClose(event);
    },
    [select, handleClose]
  );

  const handleTabKey = useCallback(
    (event: KeyboardEvent) => {
      const node = ref.current;
      if (!node) return;

      // here we query all focusable elements
      const focusableElements = node.querySelectorAll('button');
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const firstElement = focusableElements[0]!;
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const lastElement = focusableElements[focusableElements.length - 1]!;

      if (event.shiftKey) {
        // if going backward by pressing tab and firstElement is active, shift focus to last focusable element
        if (document.activeElement === firstElement) {
          lastElement.focus();
          event.preventDefault();
        }
      } else {
        // if going forward by pressing tab and lastElement is active, shift focus to first focusable element
        if (document.activeElement === lastElement) {
          firstElement.focus();
          event.preventDefault();
        }
      }
    },
    [ref]
  );

  useLayoutEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        hideModal();
      } else if (event.key === 'Tab') {
        handleTabKey(event);
      }
    };

    // Get original overflow
    const { overflow } = window.getComputedStyle(document.body);
    // Hack to enable fade in animation after mount
    setTimeout(() => setFadeIn(true), 0);
    // Prevent scrolling on mount
    document.body.style.overflow = 'hidden';
    // Listen for keydown events
    window.addEventListener('keydown', handleKeyDown, false);

    return () => {
      // Re-enable scrolling when component unmounts
      document.body.style.overflow = overflow;
      window.removeEventListener('keydown', handleKeyDown, false);
    };
  }, [hideModal, handleTabKey]);

  useLayoutEffect(
    () => setPortal(document.querySelector(container)),
    [container]
  );

  useLayoutEffect(() => {
    if (wallet) {
      connect(
        privateDataPermission || PrivateDataPermission.UponRequest,
        network || WalletAdapterNetwork.Testnet,
        allowedPrivateData ?? AllowedPrivateData.None
      ).catch((e) => {
        console.log({ e });
      });
    }
  }, [wallet, privateDataPermission, network, allowedPrivateData, connect]);

  return (
    portal &&
    createPortal(
      <div
        aria-labelledby="wallet-adapter-modal-title"
        aria-modal="true"
        className={`wallet-adapter-modal ${
          fadeIn && 'wallet-adapter-modal-fade-in'
        } ${className}`}
        ref={ref}
        role="dialog"
      >
        <div className="wallet-adapter-modal-container">
          <div className="wallet-adapter-modal-wrapper">
            <div className="wallet-adapter-modal-title">
              Connect a Wallet
              <button
                onClick={handleClose}
                className="wallet-adapter-modal-button-close"
              >
                <svg width="14" height="14">
                  <path d="M14 12.461 8.3 6.772l5.234-5.233L12.006 0 6.772 5.234 1.54 0 0 1.539l5.234 5.233L0 12.006l1.539 1.528L6.772 8.3l5.69 5.7L14 12.461z" />
                </svg>
              </button>
            </div>
            {installedWallets.length ? (
              <>
                <div className="wallet-adapter-modal-content">
                  Connect your Miden Wallet and start exploring its powerful
                  features now!
                  <hr />
                </div>
                <ul className="wallet-adapter-modal-list">
                  <span className="wallet-adapter-modal-list-section-title">
                    Recommended
                  </span>
                  <WalletListItem
                    key={getStartedWallet.adapter.name}
                    handleClick={(event) =>
                      handleWalletClick(event, getStartedWallet.adapter.name)
                    }
                    wallet={getStartedWallet}
                  />
                  {(otherInstalledWallets.length > 0 ||
                    otherWallets.length > 0) && (
                    <>
                      <hr />
                      <span className="wallet-adapter-modal-list-section-title">
                        Other wallets
                      </span>
                      {otherInstalledWallets.map((wallet) => (
                        <WalletListItem
                          key={wallet.adapter.name}
                          handleClick={(event) =>
                            handleWalletClick(event, wallet.adapter.name)
                          }
                          wallet={wallet}
                        />
                      ))}
                      {otherWallets.map((wallet) => (
                        <WalletListItem
                          key={wallet.adapter.name}
                          handleClick={(event) =>
                            handleWalletClick(event, wallet.adapter.name)
                          }
                          wallet={wallet}
                        />
                      ))}
                    </>
                  )}
                </ul>
              </>
            ) : (
              <>
                <div className="wallet-adapter-modal-middle">
                  <DiscoverMidenMessage />
                  <button
                    type="button"
                    className="wallet-adapter-modal-middle-button"
                    onClick={(event) =>
                      handleWalletClick(event, getStartedWallet.adapter.name)
                    }
                  >
                    Install Miden Wallet
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      width="25"
                      height="24"
                      viewBox="0 0 25 24"
                      fill="none"
                    >
                      <path
                        d="M10.1003 5L8.5 6.5745L13.6981 11.7L8.5 16.8255L10.1003 18.4L16.91 11.7L10.1003 5Z"
                        fill="white"
                      />
                    </svg>
                  </button>
                </div>
              </>
            )}
          </div>
        </div>
        <div
          className="wallet-adapter-modal-overlay"
          onMouseDown={handleClose}
        />
      </div>,
      portal
    )
  );
};
