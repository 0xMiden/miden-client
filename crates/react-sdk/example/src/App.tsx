import { useMiden, useSyncState, useAccounts } from "@miden-sdk/react";
import { WalletManager } from "./components/WalletManager";
import { FaucetManager } from "./components/FaucetManager";
import { TransferForm } from "./components/TransferForm";

function App() {
  const { isReady, error: initError } = useMiden();
  const { syncHeight, isSyncing, lastSyncTime, sync } = useSyncState();
  const { accounts, wallets, faucets, isLoading: accountsLoading } = useAccounts();

  if (initError) {
    return (
      <div className="card">
        <div className="status error">
          Failed to initialize Miden client: {initError.message}
        </div>
      </div>
    );
  }

  if (!isReady) {
    return (
      <div className="card">
        <div className="status loading">Initializing Miden client...</div>
      </div>
    );
  }

  return (
    <div>
      <h1>Miden React SDK Example</h1>

      {/* Sync Status */}
      <div className="card">
        <h2>Network Status</h2>
        <p>
          <strong>Block Height:</strong> {syncHeight}
        </p>
        <p>
          <strong>Last Sync:</strong>{" "}
          {lastSyncTime ? new Date(lastSyncTime).toLocaleString() : "Never"}
        </p>
        <button onClick={() => sync()} disabled={isSyncing}>
          {isSyncing ? "Syncing..." : "Sync Now"}
        </button>
      </div>

      {/* Accounts Overview */}
      <div className="card">
        <h2>Accounts</h2>
        {accountsLoading ? (
          <div className="status loading">Loading accounts...</div>
        ) : (
          <>
            <p>
              Total: {accounts.length} ({wallets.length} wallets, {faucets.length}{" "}
              faucets)
            </p>
            {accounts.length > 0 && (
              <ul className="account-list">
                {accounts.map((account) => (
                  <li key={account.id().toString()}>
                    {account.id().toString()}
                  </li>
                ))}
              </ul>
            )}
          </>
        )}
      </div>

      {/* Wallet Management */}
      <WalletManager />

      {/* Faucet Management */}
      <FaucetManager />

      {/* Transfer Form */}
      {wallets.length > 0 && faucets.length > 0 && <TransferForm />}
    </div>
  );
}

export default App;
