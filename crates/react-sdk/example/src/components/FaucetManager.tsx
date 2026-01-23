import { useState } from "react";
import { useCreateFaucet, useAccounts } from "@miden-sdk/react";

export function FaucetManager() {
  const { createFaucet, isCreating, error, reset } = useCreateFaucet();
  const { faucets } = useAccounts();

  const [tokenSymbol, setTokenSymbol] = useState("TEST");
  const [maxSupply, setMaxSupply] = useState("1000000");
  const [decimals, setDecimals] = useState("8");

  const handleCreateFaucet = async () => {
    try {
      const faucet = await createFaucet({
        tokenSymbol,
        maxSupply: BigInt(maxSupply),
        decimals: parseInt(decimals, 10),
      });
      console.log("Created faucet:", faucet.id().toString());
    } catch (err) {
      console.error("Failed to create faucet:", err);
    }
  };

  return (
    <div className="card">
      <h2>Faucet Management</h2>

      {error && (
        <div className="status error">
          Error: {error.message}
          <button onClick={reset} style={{ marginLeft: 10 }}>
            Dismiss
          </button>
        </div>
      )}

      <div style={{ marginBottom: 15 }}>
        <div style={{ marginBottom: 10 }}>
          <label>
            <strong>Token Symbol:</strong>{" "}
            <input
              type="text"
              value={tokenSymbol}
              onChange={(e) => setTokenSymbol(e.target.value.toUpperCase())}
              maxLength={4}
              style={{ width: 80 }}
            />
          </label>
        </div>

        <div style={{ marginBottom: 10 }}>
          <label>
            <strong>Max Supply:</strong>{" "}
            <input
              type="number"
              value={maxSupply}
              onChange={(e) => setMaxSupply(e.target.value)}
            />
          </label>
        </div>

        <div style={{ marginBottom: 10 }}>
          <label>
            <strong>Decimals:</strong>{" "}
            <input
              type="number"
              value={decimals}
              onChange={(e) => setDecimals(e.target.value)}
              min={0}
              max={18}
              style={{ width: 80 }}
            />
          </label>
        </div>
      </div>

      <button onClick={handleCreateFaucet} disabled={isCreating}>
        {isCreating ? "Creating Faucet..." : "Create New Faucet"}
      </button>

      {faucets.length > 0 && (
        <div style={{ marginTop: 15 }}>
          <h3>Your Faucets ({faucets.length})</h3>
          <ul className="account-list">
            {faucets.map((faucet) => (
              <li key={faucet.id().toString()}>{faucet.id().toString()}</li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
