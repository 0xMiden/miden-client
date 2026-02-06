import { useEffect, useState, type ChangeEvent, type ReactNode } from "react";
import { formatAssetAmount, formatNoteSummary, parseAssetAmount } from "@miden-sdk/react";
import { useMiden, useAccounts, useAccount, useNotes, useCreateWallet, useConsume, useSend } from "@miden-sdk/react";

const Panel = ({ title, children }: { title: string; children: ReactNode }) => (
  <div className="panel">
    <div className="label">{title}</div>
    {children}
  </div>
);

export default function App() {
  const { isReady, error } = useMiden();
  const { wallets, isLoading } = useAccounts();
  const { createWallet, isCreating } = useCreateWallet();
  const handleCreate = () => createWallet();
  const createLabel = isCreating ? "Creating..." : "Create wallet";

  if (error) return <div className="center">Error: {error.message}</div>;
  if (!isReady || isLoading) return <div className="center">{!isReady ? "Initializing..." : "Loading..."}</div>;

  const accountId = wallets[0]?.id().toString();
  if (!accountId)
    return (
      <div className="wallet">
        <h1>Wallet</h1>
        <button onClick={handleCreate} disabled={isCreating}>
          {createLabel}
        </button>
      </div>
    );

  return <Wallet accountId={accountId} />;
}

function Wallet({ accountId }: { accountId: string }) {
  const { account, assets } = useAccount(accountId);
  const { consumableNoteSummaries } = useNotes({ accountId });
  const { consume, isLoading: isConsuming } = useConsume();
  const { send, isLoading: isSending } = useSend();
  const [to, setTo] = useState("");
  const [assetId, setAssetId] = useState("");
  const [amount, setAmount] = useState("");
  const [noteType, setNoteType] = useState<"private" | "public">("private");
  const defaultAssetId = assets[0]?.assetId;
  const selectedAsset = assets.find((asset) => asset.assetId === assetId);
  const selectedDecimals = selectedAsset?.decimals;
  const hasAssets = assets.length > 0;

  useEffect(() => {
    if (!assetId && defaultAssetId) setAssetId(defaultAssetId);
  }, [assetId, defaultAssetId]);

  const handleSend = async () => {
    try {
      if (!assetId) return;
      const amt = parseAssetAmount(amount, selectedDecimals);
      await send({ from: accountId, to, assetId, amount: amt, noteType });
      setAmount("");
    } catch (error) {
      // Keep example lean; log errors for visibility
      console.error(error);
    }
  };

  const claimNote = (id: string) => () => consume({ accountId, noteIds: [id] });
  const onAssetChange = (event: ChangeEvent<HTMLSelectElement>) => setAssetId(event.target.value);
  const onNoteTypeChange = (event: ChangeEvent<HTMLSelectElement>) => setNoteType(event.target.value as "private" | "public");
  const onToChange = (event: ChangeEvent<HTMLInputElement>) => setTo(event.target.value);
  const onAmountChange = (event: ChangeEvent<HTMLInputElement>) => setAmount(event.target.value);
  const canSend = Boolean(hasAssets && to && assetId && amount);
  const sendLabel = isSending ? "Sending..." : "Send";

  return (
    <div className="wallet">
      <h1>Wallet</h1>
      <Panel title="Address">
        <div className="mono">{account?.bech32id?.() ?? "Loading..."}</div>
      </Panel>
      <Panel title="Balances">
        {assets.length === 0 ? (
          <div className="empty">None</div>
        ) : (
          <div className="list">
            {assets.map((asset) => (
              <div key={asset.assetId} className="row">
                <span className="mono">{asset.symbol ?? asset.assetId}</span>
                <span>{formatAssetAmount(asset.amount, asset.decimals)}</span>
              </div>
            ))}
          </div>
        )}
      </Panel>
      <Panel title="Unclaimed notes">
        {consumableNoteSummaries.length === 0 ? (
          <div className="empty">None</div>
        ) : (
          <div className="list">
            {consumableNoteSummaries.map((summary) => {
              const id = summary.id;
              const label = formatNoteSummary(summary);
              return (
                <div key={id} className="row">
                  <span className="mono">{label}</span>
                  <button onClick={claimNote(id)} disabled={isConsuming}>
                    Claim
                  </button>
                </div>
              );
            })}
          </div>
        )}
      </Panel>
      <Panel title="Send">
        <div className="form">
          <select value={noteType} onChange={onNoteTypeChange}>
            <option value="private">Private</option>
            <option value="public">Public</option>
          </select>
          <select value={assetId} onChange={onAssetChange} disabled={!hasAssets}>
            {hasAssets ? (
              assets.map((asset) => (
                <option key={asset.assetId} value={asset.assetId}>
                  {asset.symbol ?? asset.assetId}
                </option>
              ))
            ) : (
              <option value="">No assets</option>
            )}
          </select>
          <input placeholder="to account id" value={to} onChange={onToChange} disabled={!hasAssets} />
          <input placeholder="amount" value={amount} onChange={onAmountChange} disabled={!hasAssets} />
          <button disabled={!canSend || isSending} onClick={handleSend}>
            {sendLabel}
          </button>
        </div>
      </Panel>
    </div>
  );
}
