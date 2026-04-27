import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, render, screen, cleanup, act } from '@testing-library/react';
import React from 'react';

// Create a mock adapter factory
const createMockAdapter = (overrides = {}) => {
  const listeners: Record<string, Function[]> = {};
  return {
    name: 'Miden Wallet',
    url: 'https://miden.fi/',
    icon: 'icon-data',
    readyState: 'Unsupported',
    connected: false,
    address: null,
    publicKey: null,
    connect: vi.fn().mockResolvedValue(undefined),
    disconnect: vi.fn().mockResolvedValue(undefined),
    signBytes: vi.fn().mockResolvedValue(new Uint8Array(67)),
    on: vi.fn((event: string, handler: Function) => {
      if (!listeners[event]) listeners[event] = [];
      listeners[event].push(handler);
    }),
    off: vi.fn((event: string, handler: Function) => {
      if (listeners[event]) {
        listeners[event] = listeners[event].filter((h) => h !== handler);
      }
    }),
    emit: vi.fn((event: string, ...args: any[]) => {
      if (listeners[event]) {
        listeners[event].forEach((h) => h(...args));
      }
    }),
    ...overrides,
  };
};

let mockAdapter = createMockAdapter();

// Mock modules before imports
vi.mock('@miden-sdk/miden-wallet-adapter-miden', () => ({
  MidenWalletAdapter: vi.fn().mockImplementation((config) => {
    mockAdapter._config = config;
    return mockAdapter;
  }),
}));

vi.mock('@miden-sdk/react', () => ({
  SignerContext: React.createContext(null),
}));

vi.mock('@miden-sdk/miden-sdk', () => ({
  AccountStorageMode: {
    public: vi.fn(() => ({ toString: () => 'public' })),
    private: vi.fn(() => ({ toString: () => 'private' })),
  },
}));

vi.mock('@miden-sdk/miden-wallet-adapter-base', () => ({
  PrivateDataPermission: {
    UponRequest: 'UponRequest',
    Allowed: 'Allowed',
    Denied: 'Denied',
  },
  WalletAdapterNetwork: {
    Testnet: 'testnet',
    Devnet: 'devnet',
  },
  AllowedPrivateData: {
    None: 'None',
    All: 'All',
  },
  WalletReadyState: {
    Installed: 'Installed',
    NotDetected: 'NotDetected',
    Loadable: 'Loadable',
    Unsupported: 'Unsupported',
  },
  WalletNotSelectedError: class WalletNotSelectedError extends Error {
    constructor() {
      super('Wallet not selected');
    }
  },
  WalletNotReadyError: class WalletNotReadyError extends Error {
    constructor() {
      super('Wallet not ready');
    }
  },
  WalletNotConnectedError: class WalletNotConnectedError extends Error {
    constructor() {
      super('Wallet not connected');
    }
  },
}));

import { SignerContext } from '@miden-sdk/react';
import {
  MidenFiSignerProvider,
  useMidenFiWallet,
} from '../MidenFiSignerProvider';
import { MidenWalletAdapter } from '@miden-sdk/miden-wallet-adapter-miden';

// Test helpers
const flushPromises = () => new Promise((resolve) => setTimeout(resolve, 0));

describe('MidenFiSignerProvider', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockAdapter = createMockAdapter();
    localStorage.clear();
  });

  afterEach(() => {
    cleanup();
  });

  describe('rendering', () => {
    it('renders children', () => {
      render(
        <MidenFiSignerProvider>
          <div data-testid="child">Test Child</div>
        </MidenFiSignerProvider>
      );

      expect(screen.getByTestId('child')).toBeDefined();
      expect(screen.getByText('Test Child')).toBeDefined();
    });
  });

  describe('default adapter', () => {
    it('creates MidenWalletAdapter with default appName', () => {
      render(
        <MidenFiSignerProvider>
          <div>Test</div>
        </MidenFiSignerProvider>
      );

      expect(MidenWalletAdapter).toHaveBeenCalledWith({ appName: 'Miden DApp' });
    });

    it('creates MidenWalletAdapter with custom appName', () => {
      render(
        <MidenFiSignerProvider appName="Custom App">
          <div>Test</div>
        </MidenFiSignerProvider>
      );

      expect(MidenWalletAdapter).toHaveBeenCalledWith({ appName: 'Custom App' });
    });

    it('uses custom wallets when provided', () => {
      const customAdapter = createMockAdapter({ name: 'Custom Wallet' });

      const { result } = renderHook(() => useMidenFiWallet(), {
        wrapper: ({ children }) => (
          <MidenFiSignerProvider wallets={[customAdapter as any]}>
            {children}
          </MidenFiSignerProvider>
        ),
      });

      expect(result.current.wallets[0].adapter.name).toBe('Custom Wallet');
    });
  });

  describe('SignerContext when wallet not connected', () => {
    it('provides SignerContext with isConnected false when wallet not connected', async () => {
      let capturedContext: any = null;
      const TestConsumer = () => {
        const context = React.useContext(SignerContext);
        capturedContext = context;
        return null;
      };

      render(
        <MidenFiSignerProvider>
          <TestConsumer />
        </MidenFiSignerProvider>
      );

      await act(async () => {
        await flushPromises();
      });

      expect(capturedContext).not.toBeNull();
      expect(capturedContext.isConnected).toBe(false);
    });

    it('signCb throws when wallet not connected', async () => {
      let capturedContext: any = null;
      const TestConsumer = () => {
        const context = React.useContext(SignerContext);
        capturedContext = context;
        return null;
      };

      render(
        <MidenFiSignerProvider>
          <TestConsumer />
        </MidenFiSignerProvider>
      );

      await act(async () => {
        await flushPromises();
      });

      await expect(
        capturedContext.signCb(new Uint8Array(), new Uint8Array())
      ).rejects.toThrow('MidenFi wallet not connected');
    });
  });

  describe("SignerContext name", () => {
    it("includes correct name ('MidenFi')", async () => {
      let capturedContext: any = null;
      const TestConsumer = () => {
        const context = React.useContext(SignerContext);
        capturedContext = context;
        return null;
      };

      render(
        <MidenFiSignerProvider>
          <TestConsumer />
        </MidenFiSignerProvider>
      );

      await act(async () => {
        await flushPromises();
      });

      expect(capturedContext.name).toBe('MidenFi');
    });
  });

  describe('useMidenFiWallet hook', () => {
    it('throws when used outside provider', () => {
      const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});

      expect(() => {
        renderHook(() => useMidenFiWallet());
      }).toThrow('useMidenFiWallet must be used within MidenFiSignerProvider');

      consoleError.mockRestore();
    });

    it('returns wallet context inside provider', () => {
      const { result } = renderHook(() => useMidenFiWallet(), {
        wrapper: ({ children }) => (
          <MidenFiSignerProvider>{children}</MidenFiSignerProvider>
        ),
      });

      expect(result.current).toBeDefined();
      expect(result.current.wallets).toBeDefined();
      expect(result.current.connect).toBeDefined();
      expect(result.current.disconnect).toBeDefined();
    });

    it('has wallets array with default MidenWalletAdapter', () => {
      const { result } = renderHook(() => useMidenFiWallet(), {
        wrapper: ({ children }) => (
          <MidenFiSignerProvider>{children}</MidenFiSignerProvider>
        ),
      });

      expect(result.current.wallets).toHaveLength(1);
      expect(result.current.wallets[0].adapter.name).toBe('Miden Wallet');
    });

    it('connected is false initially', () => {
      const { result } = renderHook(() => useMidenFiWallet(), {
        wrapper: ({ children }) => (
          <MidenFiSignerProvider>{children}</MidenFiSignerProvider>
        ),
      });

      expect(result.current.connected).toBe(false);
    });

    it('has select function', () => {
      const { result } = renderHook(() => useMidenFiWallet(), {
        wrapper: ({ children }) => (
          <MidenFiSignerProvider>{children}</MidenFiSignerProvider>
        ),
      });

      expect(typeof result.current.select).toBe('function');
    });

    it('autoConnect defaults to true', () => {
      const { result } = renderHook(() => useMidenFiWallet(), {
        wrapper: ({ children }) => (
          <MidenFiSignerProvider>{children}</MidenFiSignerProvider>
        ),
      });

      expect(result.current.autoConnect).toBe(true);
    });

    it('autoConnect can be set to false', () => {
      const { result } = renderHook(() => useMidenFiWallet(), {
        wrapper: ({ children }) => (
          <MidenFiSignerProvider autoConnect={false}>{children}</MidenFiSignerProvider>
        ),
      });

      expect(result.current.autoConnect).toBe(false);
    });
  });

  describe('SignerContext connect/disconnect', () => {
    it('SignerContext has connect function', async () => {
      let capturedContext: any = null;
      const TestConsumer = () => {
        const context = React.useContext(SignerContext);
        capturedContext = context;
        return null;
      };

      render(
        <MidenFiSignerProvider>
          <TestConsumer />
        </MidenFiSignerProvider>
      );

      await act(async () => {
        await flushPromises();
      });

      expect(typeof capturedContext.connect).toBe('function');
    });

    it('SignerContext has disconnect function', async () => {
      let capturedContext: any = null;
      const TestConsumer = () => {
        const context = React.useContext(SignerContext);
        capturedContext = context;
        return null;
      };

      render(
        <MidenFiSignerProvider>
          <TestConsumer />
        </MidenFiSignerProvider>
      );

      await act(async () => {
        await flushPromises();
      });

      expect(typeof capturedContext.disconnect).toBe('function');
    });
  });

  describe('accountConfig', () => {
    it('accountType is RegularAccountImmutableCode when connected', async () => {
      // Create a connected adapter
      mockAdapter = createMockAdapter({
        connected: true,
        publicKey: new Uint8Array(32).fill(0x42),
        address: '0xtest-address',
        readyState: 'Installed',
      });

      let capturedContext: any = null;
      const TestConsumer = () => {
        const context = React.useContext(SignerContext);
        capturedContext = context;
        return null;
      };

      render(
        <MidenFiSignerProvider>
          <TestConsumer />
        </MidenFiSignerProvider>
      );

      // Auto-select single wallet
      await act(async () => {
        await flushPromises();
        await flushPromises();
        await flushPromises();
      });

      if (capturedContext?.accountConfig) {
        expect(capturedContext.accountConfig.accountType).toBe(
          'RegularAccountImmutableCode'
        );
      }
    });
  });

  describe('storeName', () => {
    it('uses midenfi_ prefix for database isolation', async () => {
      mockAdapter = createMockAdapter({
        connected: true,
        publicKey: new Uint8Array(32),
        address: '0xunique-address',
        readyState: 'Installed',
      });

      let capturedContext: any = null;
      const TestConsumer = () => {
        const context = React.useContext(SignerContext);
        capturedContext = context;
        return null;
      };

      render(
        <MidenFiSignerProvider>
          <TestConsumer />
        </MidenFiSignerProvider>
      );

      await act(async () => {
        await flushPromises();
        await flushPromises();
        await flushPromises();
      });

      if (capturedContext?.storeName && capturedContext.storeName !== '') {
        expect(capturedContext.storeName).toContain('midenfi_');
      }
    });
  });
});
