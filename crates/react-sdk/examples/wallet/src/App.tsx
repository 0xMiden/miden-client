import { useEffect, useState } from "react";
import {
  useMiden,
  useAccounts,
  useAccount,
  useNotes,
  useCreateWallet,
  useConsume,
  useSend,
} from "@miden-sdk/react";

export default function App() {
  const { isReady, error } = useMiden();
  const { wallets, isLoading } = useAccounts();
  const { createWallet, isCreating } = useCreateWallet();

  if (error) return <div className="center">Error: {error.message}</div>;
  if (!isReady) return <div className="center">Initializing...</div>;
  if (isLoading) return <div className="center">Loading...</div>;

  const accountId = wallets[0]?.id().toString();
  if (!accountId) {
    return (
      <div className="wallet">
        <h1>Wallet</h1>
        <button onClick={() => createWallet()} disabled={isCreating}>
          {isCreating ? "Creating..." : "Create wallet"}
        </button>
      </div>
    );
  }

  return <Wallet accountId={accountId} />;
}

function Wallet({ accountId }: { accountId: string }) {
  const { account, assets } = useAccount(accountId);
  const { consumableNotes } = useNotes({ accountId });
  const { consume, isLoading: isConsuming } = useConsume();
  const { send, isLoading: isSending } = useSend();
  const [to, setTo] = useState("");
  const [assetId, setAssetId] = useState("");
  const [amount, setAmount] = useState("");
  const hasAssets = assets.length > 0;

  useEffect(() => {
    if (!assetId && hasAssets) {
      setAssetId(assets[0].assetId);
    }
  }, [assetId, assets, hasAssets]);

  const handleSend = async () => {
    try {
      if (!assetId) return;
      await send({
        from: accountId,
        to,
        assetId,
        amount: BigInt(amount),
      });
      setAmount("");
    } catch {
      // Keep example lean; ignore errors here.
    }
  };

  const canSend = hasAssets && to && assetId && amount;

  const bech32Address = account?.bech32id?.() ?? "Loading...";

  return (
    <div className="wallet">
      <h1>Wallet</h1>

      <div className="panel">
        <div className="label">Address</div>
        <div className="mono">{bech32Address}</div>
      </div>

      <div className="panel">
        <div className="label">Balances</div>
        {assets.length === 0 ? (
          <div className="empty">None</div>
        ) : (
          <div className="list">
            {assets.map((asset) => (
              <div key={asset.assetId} className="row">
                <span className="mono">{asset.assetId}</span>
                <span>{asset.amount.toString()}</span>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="panel">
        <div className="label">Unclaimed notes</div>
        {consumableNotes.length === 0 ? (
          <div className="empty">None</div>
        ) : (
          <div className="list">
            {consumableNotes.map((note) => {
              const id = note.inputNoteRecord().id().toString();
              return (
                <div key={id} className="row">
                  <span className="mono">{id}</span>
                  <button
                    onClick={() => consume({ accountId, noteIds: [id] })}
                    disabled={isConsuming}
                  >
                    Claim
                  </button>
                </div>
              );
            })}
          </div>
        )}
      </div>

      <div className="panel">
        <div className="label">Send</div>
        <div className="form">
          <select
            value={assetId}
            onChange={(event) => setAssetId(event.target.value)}
            disabled={!hasAssets}
          >
            {hasAssets ? (
              assets.map((asset) => (
                <option key={asset.assetId} value={asset.assetId}>
                  {asset.assetId}
                </option>
              ))
            ) : (
              <option value="">No assets</option>
            )}
          </select>
          <input
            placeholder="to account id"
            value={to}
            onChange={(event) => setTo(event.target.value)}
            disabled={!hasAssets}
          />
          <input
            placeholder="amount"
            value={amount}
            onChange={(event) => setAmount(event.target.value)}
            disabled={!hasAssets}
          />
          <button disabled={!canSend || isSending} onClick={handleSend}>
            {isSending ? "Sending..." : "Send"}
          </button>
        </div>
      </div>
    </div>
  );
}
