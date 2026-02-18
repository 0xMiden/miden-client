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
