import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  waitForWalletDetection,
  WalletAdapterLike,
} from "../../utils/walletDetection";

function createMockAdapter(
  initialState: string = "NotDetected"
): WalletAdapterLike & { emit: (state: string) => void } {
  const listeners = new Set<(state: string) => void>();
  return {
    readyState: initialState,
    on(_event: "readyStateChange", cb: (state: string) => void) {
      listeners.add(cb);
    },
    off(_event: "readyStateChange", cb: (state: string) => void) {
      listeners.delete(cb);
    },
    emit(state: string) {
      this.readyState = state;
      for (const cb of listeners) cb(state);
    },
  };
}

describe("waitForWalletDetection", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("should resolve immediately if already installed", async () => {
    const adapter = createMockAdapter("Installed");
    await waitForWalletDetection(adapter);
  });

  it("should resolve when readyStateChange fires with Installed", async () => {
    const adapter = createMockAdapter("NotDetected");
    const promise = waitForWalletDetection(adapter);

    adapter.emit("Installed");

    await promise;
  });

  it("should ignore non-Installed state changes", async () => {
    const adapter = createMockAdapter("NotDetected");
    const promise = waitForWalletDetection(adapter);

    adapter.emit("Loadable");
    // Should still be pending — advance timer to trigger timeout
    vi.advanceTimersByTime(5000);

    await expect(promise).rejects.toThrow("not detected within 5000ms");
  });

  it("should reject on timeout with default 5000ms", async () => {
    const adapter = createMockAdapter("NotDetected");
    const promise = waitForWalletDetection(adapter);

    vi.advanceTimersByTime(5000);

    await expect(promise).rejects.toThrow("not detected within 5000ms");
    await expect(promise).rejects.toThrow(
      "Is the browser extension installed and enabled?"
    );
  });

  it("should reject on custom timeout", async () => {
    const adapter = createMockAdapter("NotDetected");
    const promise = waitForWalletDetection(adapter, 2000);

    vi.advanceTimersByTime(2000);

    await expect(promise).rejects.toThrow("not detected within 2000ms");
  });

  it("should clean up listener on successful detection", async () => {
    const adapter = createMockAdapter("NotDetected");
    const offSpy = vi.spyOn(adapter, "off");
    const promise = waitForWalletDetection(adapter);

    adapter.emit("Installed");
    await promise;

    expect(offSpy).toHaveBeenCalledWith(
      "readyStateChange",
      expect.any(Function)
    );
  });

  it("should clean up listener on timeout", async () => {
    const adapter = createMockAdapter("NotDetected");
    const offSpy = vi.spyOn(adapter, "off");
    const promise = waitForWalletDetection(adapter);

    vi.advanceTimersByTime(5000);

    await expect(promise).rejects.toThrow();
    expect(offSpy).toHaveBeenCalledWith(
      "readyStateChange",
      expect.any(Function)
    );
  });

  it("should handle race condition where readyState changes before listener is registered", async () => {
    // Simulate adapter that becomes Installed between the initial check and listener setup
    const adapter = createMockAdapter("NotDetected");

    // Override on() to change readyState as a side effect (simulating the race)
    const originalOn = adapter.on.bind(adapter);
    adapter.on = (event: "readyStateChange", cb: (state: string) => void) => {
      originalOn(event, cb);
      // Simulate the adapter becoming ready right after listener registration
      adapter.readyState = "Installed";
    };

    await waitForWalletDetection(adapter);
  });
});
