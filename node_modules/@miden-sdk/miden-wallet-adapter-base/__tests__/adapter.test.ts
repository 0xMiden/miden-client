import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  WalletReadyState,
  BaseWalletAdapter,
  scopePollingDetectionStrategy,
} from '../adapter';

describe('WalletReadyState enum', () => {
  it('has correct values', () => {
    expect(WalletReadyState.Installed).toBe('Installed');
    expect(WalletReadyState.NotDetected).toBe('NotDetected');
    expect(WalletReadyState.Loadable).toBe('Loadable');
    expect(WalletReadyState.Unsupported).toBe('Unsupported');
  });
});

describe('BaseWalletAdapter', () => {
  // Concrete subclass for testing
  class TestAdapter extends BaseWalletAdapter {
    name = 'Test' as any;
    url = 'https://test.com';
    icon = 'icon';
    readyState = WalletReadyState.Installed;
    address: string | null = null;
    publicKey: Uint8Array | null = null;
    connecting = false;
    supportedTransactionVersions = null;

    async connect() {}
    async disconnect() {}
  }

  it('connected returns true when address is set', () => {
    const adapter = new TestAdapter();
    adapter.address = '0xabc';
    expect(adapter.connected).toBe(true);
  });

  it('connected returns false when address is null', () => {
    const adapter = new TestAdapter();
    expect(adapter.connected).toBe(false);
  });
});

describe('scopePollingDetectionStrategy', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('calls detect immediately and cleans up on detection', () => {
    const detect = vi.fn().mockReturnValue(true);
    scopePollingDetectionStrategy(detect);
    expect(detect).toHaveBeenCalled();
  });

  it('sets up polling interval when not immediately detected', () => {
    let callCount = 0;
    const detect = vi.fn(() => {
      callCount++;
      return callCount >= 3;
    });

    scopePollingDetectionStrategy(detect);
    // First call is immediate (returns false)
    expect(detect).toHaveBeenCalledTimes(1);

    // Advance timer to trigger interval
    vi.advanceTimersByTime(1000);
    expect(detect).toHaveBeenCalledTimes(2);

    vi.advanceTimersByTime(1000);
    expect(detect).toHaveBeenCalledTimes(3);

    // After detection (callCount >= 3), interval should be cleared
    vi.advanceTimersByTime(1000);
    expect(detect).toHaveBeenCalledTimes(3);
  });
});
