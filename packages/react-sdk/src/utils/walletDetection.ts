/** Minimal adapter shape (duck-typed, no dependency on wallet-adapter-base) */
export interface WalletAdapterLike {
  readyState: string;
  on(event: "readyStateChange", cb: (state: string) => void): void;
  off(event: "readyStateChange", cb: (state: string) => void): void;
}

/**
 * Wait for a wallet adapter to reach "Installed" readyState.
 * Returns immediately if already installed. Otherwise listens
 * for readyStateChange events with a timeout.
 *
 * @example
 * ```ts
 * const adapter = wallets[0].adapter;
 * await waitForWalletDetection(adapter);        // default 5s timeout
 * await waitForWalletDetection(adapter, 10000); // 10s timeout
 * ```
 */
export async function waitForWalletDetection(
  adapter: WalletAdapterLike,
  timeoutMs = 5000,
): Promise<void> {
  if (adapter.readyState === "Installed") return;

  return new Promise<void>((resolve, reject) => {
    let settled = false;

    const settle = () => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      adapter.off("readyStateChange", onReady);
      resolve();
    };

    const timer = setTimeout(() => {
      if (settled) return;
      settled = true;
      adapter.off("readyStateChange", onReady);
      reject(
        new Error(
          `Wallet extension not detected within ${timeoutMs}ms. ` +
            "Is the browser extension installed and enabled?",
        ),
      );
    }, timeoutMs);

    const onReady = (state: string) => {
      if (state === "Installed") settle();
    };
    adapter.on("readyStateChange", onReady);

    // Re-check in case readyState changed between the if-check and listener registration
    if (adapter.readyState === "Installed") settle();
  });
}
