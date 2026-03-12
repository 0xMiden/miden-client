import { AccountId, Address } from "@miden-sdk/miden-sdk";
import type { AccountId as AccountIdType } from "@miden-sdk/miden-sdk";

const normalizeAccountIdInput = (value: string): string =>
  value.trim().replace(/^miden:/i, "");

const isBech32Input = (value: string): boolean =>
  value.startsWith("m") || value.startsWith("M");

const normalizeHexInput = (value: string): string =>
  value.startsWith("0x") || value.startsWith("0X") ? value : `0x${value}`;

const parseAccountIdFromString = (value: string): AccountIdType => {
  if (isBech32Input(value)) {
    try {
      return Address.fromBech32(value).accountId();
    } catch {
      return AccountId.fromBech32(value);
    }
  }

  return AccountId.fromHex(normalizeHexInput(value));
};

export const parseAccountId = (
  value: string | AccountIdType
): AccountIdType => {
  if (typeof value !== "string") {
    return value;
  }

  const normalized = normalizeAccountIdInput(value);
  return parseAccountIdFromString(normalized);
};

/**
 * Check if an account ID represents a faucet.
 * Faucet IDs have bits 61..=60 == 0b10 (Fungible Faucet) or 0b11 (Non-fungible Faucet).
 */
export function isFaucetId(accountId: unknown): boolean {
  try {
    let hex =
      typeof (accountId as { toHex?: () => string }).toHex === "function"
        ? (accountId as { toHex: () => string }).toHex()
        : String(accountId);

    if (hex.startsWith("0x") || hex.startsWith("0X")) {
      hex = hex.slice(2);
    }

    // Account type is in bits 61..60 of the u64:
    // 0b00 = Regular account (off-chain)
    // 0b01 = Regular account (on-chain)
    // 0b10 = Fungible faucet
    // 0b11 = Non-fungible faucet
    const firstByte = parseInt(hex.slice(0, 2), 16);
    const accountType = (firstByte >> 4) & 0b11;

    return accountType === 0b10 || accountType === 0b11;
  } catch {
    return false;
  }
}

export const parseAddress = (
  value: string,
  accountId?: AccountIdType
): Address => {
  const normalized = normalizeAccountIdInput(value);

  if (isBech32Input(normalized)) {
    try {
      return Address.fromBech32(normalized);
    } catch {
      const resolvedAccountId = accountId ?? AccountId.fromBech32(normalized);
      return Address.fromAccountId(resolvedAccountId, "BasicWallet");
    }
  }

  const resolvedAccountId =
    accountId ?? AccountId.fromHex(normalizeHexInput(normalized));
  return Address.fromAccountId(resolvedAccountId, "BasicWallet");
};
