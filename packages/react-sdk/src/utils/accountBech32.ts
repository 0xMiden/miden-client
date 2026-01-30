import {
  Account,
  AccountId,
  AccountInterface,
  Address,
  NetworkId,
} from "@miden-sdk/miden-sdk";
import { useMidenStore } from "../store/MidenStore";
import { parseAccountId } from "./accountParsing";

type AccountPrototype = {
  bech32id?: () => string;
};

const inferNetworkId = (): NetworkId => {
  const { rpcUrl } = useMidenStore.getState().config;
  if (!rpcUrl) {
    return NetworkId.testnet();
  }

  const url = rpcUrl.toLowerCase();
  if (url.includes("devnet") || url.includes("mdev")) {
    return NetworkId.devnet();
  }
  if (url.includes("mainnet")) {
    return NetworkId.mainnet();
  }
  if (url.includes("testnet") || url.includes("mtst")) {
    return NetworkId.testnet();
  }

  return NetworkId.testnet();
};

const toBech32FromAccountId = (id: AccountId): string => {
  try {
    const address = Address.fromAccountId(id, "BasicWallet");
    return address.toBech32(inferNetworkId());
  } catch {
    // Fall through to AccountId conversion or string fallback.
  }

  try {
    const maybeBech32 = id.toBech32?.(
      inferNetworkId(),
      AccountInterface.BasicWallet
    );
    if (typeof maybeBech32 === "string") {
      return maybeBech32;
    }
  } catch {
    // Fall through to string fallback.
  }

  return id.toString();
};

const defineBech32 = (target: AccountPrototype | Account): boolean => {
  try {
    Object.defineProperty(target, "bech32id", {
      value: function bech32id() {
        try {
          const id = this.id?.();
          if (id) {
            return toBech32FromAccountId(id);
          }
        } catch {
          // Fall through to string-based conversion.
        }

        const fallback =
          typeof this.toString === "function" ? this.toString() : "";
        return fallback ? toBech32AccountId(fallback) : "";
      },
    });
    return true;
  } catch {
    return false;
  }
};

export const installAccountBech32 = () => {
  const proto = Account.prototype as AccountPrototype;
  if (proto.bech32id) {
    return;
  }

  defineBech32(proto);
};

export const ensureAccountBech32 = (account?: Account | null) => {
  if (!account) {
    return;
  }

  if (typeof (account as AccountPrototype).bech32id === "function") {
    return;
  }

  const proto = Object.getPrototypeOf(account) as AccountPrototype | null;
  if (proto?.bech32id) {
    return;
  }

  if (proto && defineBech32(proto)) {
    return;
  }

  defineBech32(account as unknown as AccountPrototype);
};

export const toBech32AccountId = (accountId: string): string => {
  try {
    const id = parseAccountId(accountId);
    return toBech32FromAccountId(id);
  } catch {
    return accountId;
  }
};
