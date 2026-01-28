import type {
  ConsumableNoteRecord,
  InputNoteRecord,
} from "@miden-sdk/miden-sdk";
import type { AssetMetadata, NoteAsset, NoteSummary } from "../types";
import { toBech32AccountId } from "./accountBech32";

const getInputNoteRecord = (
  note: ConsumableNoteRecord | InputNoteRecord
): InputNoteRecord => {
  const maybeConsumable = note as ConsumableNoteRecord;
  if (typeof maybeConsumable.inputNoteRecord === "function") {
    return maybeConsumable.inputNoteRecord();
  }
  return note as InputNoteRecord;
};

export const getNoteSummary = (
  note: ConsumableNoteRecord | InputNoteRecord,
  getAssetMetadata?: (assetId: string) => AssetMetadata | undefined
): NoteSummary | null => {
  try {
    const record = getInputNoteRecord(note);
    const id = record.id().toString();
    const assets: NoteAsset[] = [];

    try {
      const details = record.details();
      const assetsList = details?.assets?.().fungibleAssets?.() ?? [];
      for (const asset of assetsList) {
        const assetId = asset.faucetId().toString();
        const metadata = getAssetMetadata?.(assetId);
        assets.push({
          assetId,
          amount: BigInt(asset.amount() as number | bigint),
          symbol: metadata?.symbol,
          decimals: metadata?.decimals,
        });
      }
    } catch {
      // Keep assets empty if details are unavailable.
    }

    const metadata = record.metadata?.();
    const senderHex = metadata?.sender?.()?.toString?.();
    const sender = senderHex ? toBech32AccountId(senderHex) : undefined;

    return { id, assets, sender };
  } catch {
    return null;
  }
};

export const formatNoteSummary = (
  summary: NoteSummary,
  formatAsset: (asset: NoteAsset) => string = (asset) =>
    `${asset.amount.toString()} ${asset.symbol ?? asset.assetId}`
): string => {
  if (!summary.assets.length) {
    return summary.id;
  }

  const assetsText = summary.assets.map(formatAsset).join(" + ");
  return summary.sender ? `${assetsText} from ${summary.sender}` : assetsText;
};
