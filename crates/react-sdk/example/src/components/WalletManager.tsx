import { useState } from "react";
import { useCreateWallet, useAccounts } from "@miden-sdk/react";

export function WalletManager() {
  const { createWallet, isCreating, error, reset } = useCreateWallet();
  const { wallets } = useAccounts();
  const [storageMode, setStorageMode] = useState<"private" | "public">("private");

  const handleCreateWallet = async () => {
    try {
      const wallet = await createWallet({ storageMode });
      console.log("Created wallet:", wallet.id().toString());
    } catch (err) {
      console.error("Failed to create wallet:", err);
    }
  };

  return (
    <div className="card">
      <h2>Wallet Management</h2>

      {error && (
        <div className="status error">
          Error: {error.message}
          <button onClick={reset} style={{ marginLeft: 10 }}>
            Dismiss
          </button>
        </div>
      )}

      <div style={{ marginBottom: 15 }}>
        <label>
          <strong>Storage Mode:</strong>{" "}
          <select
            value={storageMode}
            onChange={(e) => setStorageMode(e.target.value as "private" | "public")}
          >
            <option value="private">Private (default)</option>
            <option value="public">Public</option>
          </select>
        </label>
      </div>

      <button onClick={handleCreateWallet} disabled={isCreating}>
        {isCreating ? "Creating Wallet..." : "Create New Wallet"}
      </button>

      {wallets.length > 0 && (
        <div style={{ marginTop: 15 }}>
          <h3>Your Wallets ({wallets.length})</h3>
          <ul className="account-list">
            {wallets.map((wallet) => (
              <li key={wallet.id().toString()}>{wallet.id().toString()}</li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
