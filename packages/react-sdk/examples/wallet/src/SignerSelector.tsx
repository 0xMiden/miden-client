import { useState } from "react";
import { useMultiSigner } from "@miden-sdk/react";

export function SignerSelector({
  multiSigner,
  onUseLocal,
}: {
  multiSigner: ReturnType<typeof useMultiSigner>;
  onUseLocal: () => void;
}) {
  const [error, setError] = useState<string | null>(null);

  const handleConnect = async (name: string) => {
    if (!multiSigner) return;
    setError(null);
    try {
      await multiSigner.connectSigner(name);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Connection failed");
    }
  };

  return (
    <div className="wallet">
      <h1>Connect Wallet</h1>
      <div className="form">
        {multiSigner?.signers.map((s) => (
          <button key={s.name} onClick={() => handleConnect(s.name)}>
            {s.name}
          </button>
        ))}
        <button onClick={onUseLocal}>Use Local Keystore</button>
      </div>
      {error && <div className="center">Error: {error}</div>}
    </div>
  );
}
