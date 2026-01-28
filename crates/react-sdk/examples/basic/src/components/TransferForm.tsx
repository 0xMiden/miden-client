import { useState } from "react";
import { useSend, useMint, useAccounts } from "@miden-sdk/react";

export function TransferForm() {
  const { wallets, faucets } = useAccounts();
  const {
    send,
    isLoading: isSending,
    stage: sendStage,
    error: sendError,
    reset: resetSend,
  } = useSend();
  const {
    mint,
    isLoading: isMinting,
    stage: mintStage,
    error: mintError,
    reset: resetMint,
  } = useMint();

  const [selectedWallet, setSelectedWallet] = useState("");
  const [selectedAsset, setSelectedAsset] = useState("");
  const [recipientAddress, setRecipientAddress] = useState("");
  const [amount, setAmount] = useState("100");

  // Auto-select first wallet/faucet if not selected
  if (!selectedWallet && wallets.length > 0) {
    setSelectedWallet(wallets[0].id().toString());
  }
  if (!selectedAsset && faucets.length > 0) {
    setSelectedAsset(faucets[0].id().toString());
  }

  const handleMint = async () => {
    if (!selectedWallet || !selectedAsset) return;

    try {
      const result = await mint({
        targetAccountId: selectedWallet,
        faucetId: selectedAsset,
        amount: BigInt(amount),
      });
      console.log("Minted tokens, tx:", result.transactionId);
    } catch (err) {
      console.error("Mint failed:", err);
    }
  };

  const handleSend = async () => {
    if (!selectedWallet || !selectedAsset || !recipientAddress) return;

    try {
      const result = await send({
        from: selectedWallet,
        to: recipientAddress,
        assetId: selectedAsset,
        amount: BigInt(amount),
      });
      console.log("Sent tokens, tx:", result.transactionId);
    } catch (err) {
      console.error("Send failed:", err);
    }
  };

  const error = sendError || mintError;
  const resetError = () => {
    resetSend();
    resetMint();
  };

  return (
    <div className="card">
      <h2>Token Operations</h2>

      {error && (
        <div className="status error">
          Error: {error.message}
          <button onClick={resetError} style={{ marginLeft: 10 }}>
            Dismiss
          </button>
        </div>
      )}

      <div style={{ marginBottom: 15 }}>
        <div style={{ marginBottom: 10 }}>
          <label>
            <strong>Wallet:</strong>{" "}
            <select
              value={selectedWallet}
              onChange={(e) => setSelectedWallet(e.target.value)}
              style={{ width: 300, fontFamily: "monospace" }}
            >
              {wallets.map((wallet) => (
                <option
                  key={wallet.id().toString()}
                  value={wallet.id().toString()}
                >
                  {wallet.id().toString()}
                </option>
              ))}
            </select>
          </label>
        </div>

        <div style={{ marginBottom: 10 }}>
          <label>
            <strong>Asset:</strong>{" "}
            <select
              value={selectedAsset}
              onChange={(e) => setSelectedAsset(e.target.value)}
              style={{ width: 300, fontFamily: "monospace" }}
            >
              {faucets.map((faucet) => (
                <option
                  key={faucet.id().toString()}
                  value={faucet.id().toString()}
                >
                  {faucet.id().toString()}
                </option>
              ))}
            </select>
          </label>
        </div>

        <div style={{ marginBottom: 10 }}>
          <label>
            <strong>Amount:</strong>{" "}
            <input
              type="number"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              style={{ width: 150 }}
            />
          </label>
        </div>
      </div>

      <h3>Mint Tokens</h3>
      <p style={{ color: "#666", fontSize: 14 }}>
        Mint tokens from the selected asset (faucet) to the selected wallet.
      </p>
      <button
        onClick={handleMint}
        disabled={isMinting || !selectedWallet || !selectedAsset}
      >
        {isMinting ? `Minting (${mintStage})...` : "Mint Tokens"}
      </button>

      <h3 style={{ marginTop: 20 }}>Send Tokens</h3>
      <p style={{ color: "#666", fontSize: 14 }}>
        Send tokens from the selected wallet to another address.
      </p>
      <div style={{ marginBottom: 10 }}>
        <label>
          <strong>Recipient Address:</strong>{" "}
          <input
            type="text"
            value={recipientAddress}
            onChange={(e) => setRecipientAddress(e.target.value)}
            placeholder="0x..."
            style={{ width: 300, fontFamily: "monospace" }}
          />
        </label>
      </div>
      <button
        onClick={handleSend}
        disabled={
          isSending || !selectedWallet || !selectedAsset || !recipientAddress
        }
      >
        {isSending ? `Sending (${sendStage})...` : "Send Tokens"}
      </button>
    </div>
  );
}
