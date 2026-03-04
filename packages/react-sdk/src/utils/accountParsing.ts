import { AccountId, Address } from "@miden-sdk/miden-sdk";
import type {
  AccountId as AccountIdType,
  Account,
  AccountHeader,
} from "@miden-sdk/miden-sdk";

/** Account reference — any account ID form accepted by the React SDK hooks. */
export type AccountRef = string | AccountIdType | Account | AccountHeader;

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

export const parseAccountId = (value: AccountRef): AccountIdType => {
  if (typeof value === "string") {
    return parseAccountIdFromString(normalizeAccountIdInput(value));
  }
  // Account or AccountHeader — extract their AccountId via .id()
  if (typeof (value as Account | AccountHeader).id === "function") {
    return (value as Account | AccountHeader).id();
  }
  // Already an AccountId
  return value as AccountIdType;
};

export const parseAddress = (
  value: AccountRef,
  accountId?: AccountIdType
): Address => {
  if (typeof value !== "string") {
    // Non-string: resolve the AccountId and wrap in Address
    const resolvedId = accountId ?? parseAccountId(value);
    return Address.fromAccountId(resolvedId, "BasicWallet");
  }

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
