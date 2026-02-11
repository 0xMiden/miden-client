import { useCallback, useState } from "react";
import { useMiden } from "../context/MidenProvider";
import { useMidenStore } from "../store/MidenStore";
import { AccountFile } from "@miden-sdk/miden-sdk";
import type {
  Account,
  AccountId as AccountIdType,
  AccountFile as AccountFileType,
  _WebClient as WebClient,
} from "@miden-sdk/miden-sdk";
import type { ImportAccountOptions } from "../types";
import { DEFAULTS } from "../types";
import { parseAccountId } from "../utils/accountParsing";
import { runExclusiveDirect } from "../utils/runExclusive";
import { ensureAccountBech32 } from "../utils/accountBech32";

export interface UseImportAccountResult {
  /** Import an existing account into the client */
  importAccount: (options: ImportAccountOptions) => Promise<Account>;
  /** The imported account */
  account: Account | null;
  /** Whether an import is in progress */
  isImporting: boolean;
  /** Error if import failed */
  error: Error | null;
  /** Reset the hook state */
  reset: () => void;
}

type AccountFileWithAccount = AccountFileType & {
  account: () => Account;
  accountId?: () => AccountIdType;
};

/**
 * Hook to import existing accounts into the client.
 *
 * @example
 * ```tsx
 * function ImportAccountButton({ accountId }: { accountId: string }) {
 *   const { importAccount, isImporting } = useImportAccount();
 *
 *   const handleImport = async () => {
 *     const account = await importAccount({
 *       type: "id",
 *       accountId,
 *     });
 *     console.log("Imported:", account.id().toString());
 *   };
 *
 *   return (
 *     <button onClick={handleImport} disabled={isImporting}>
 *       {isImporting ? "Importing..." : "Import Account"}
 *     </button>
 *   );
 * }
 * ```
 */
export function useImportAccount(): UseImportAccountResult {
  const { client, isReady, runExclusive } = useMiden();
  const runExclusiveSafe = runExclusive ?? runExclusiveDirect;
  const setAccounts = useMidenStore((state) => state.setAccounts);

  const [account, setAccount] = useState<Account | null>(null);
  const [isImporting, setIsImporting] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const importAccount = useCallback(
    async (options: ImportAccountOptions): Promise<Account> => {
      if (!client || !isReady) {
        throw new Error("Miden client is not ready");
      }

      setIsImporting(true);
      setError(null);

      try {
        type AccountHeaders = Awaited<ReturnType<WebClient["getAccounts"]>>;
        let accountsAfter: AccountHeaders | null = null;

        const imported = await runExclusiveSafe(async () => {
          switch (options.type) {
            case "file": {
              const accountsBefore = await client.getAccounts();
              const accountFile = await resolveAccountFile(options.file);
              const accountFileWithAccount =
                accountFile as AccountFileWithAccount;
              const fileBytes = getAccountFileBytes(
                accountFileWithAccount,
                options.file
              );
              const accountFromFile =
                typeof accountFileWithAccount.account === "function"
                  ? accountFileWithAccount.account()
                  : null;
              const accountIdFromFile =
                accountFromFile === null &&
                typeof accountFileWithAccount.accountId === "function"
                  ? accountFileWithAccount.accountId()
                  : null;

              try {
                await client.importAccountFile(accountFile);
              } catch (err) {
                const message =
                  err instanceof Error ? err.message : String(err);
                if (!message.includes("already being tracked")) {
                  throw err;
                }
              }

              accountsAfter = await client.getAccounts();

              if (accountFromFile) {
                return accountFromFile;
              }

              const beforeIds = new Set(
                accountsBefore.map((account) => account.id().toString())
              );
              const newAccountHeader = accountsAfter.find(
                (account) => !beforeIds.has(account.id().toString())
              );
              const accountId = accountIdFromFile ?? newAccountHeader?.id();
              if (accountId) {
                const fetchedAccount = await client.getAccount(accountId);
                if (fetchedAccount) {
                  return fetchedAccount;
                }
              }

              if (fileBytes) {
                for (const header of accountsAfter) {
                  const exported = await client.exportAccountFile(header.id());
                  const exportedBytes = getAccountFileBytes(exported, exported);
                  if (exportedBytes && bytesEqual(exportedBytes, fileBytes)) {
                    const fetchedAccount = await client.getAccount(header.id());
                    if (fetchedAccount) {
                      return fetchedAccount;
                    }
                  }
                }
              }

              throw new Error("Account not found after import");
            }
            case "id": {
              const accountId = resolveAccountId(options.accountId);
              await client.importAccountById(accountId);
              const fetchedAccount = await client.getAccount(accountId);
              if (!fetchedAccount) {
                throw new Error("Account not found after import");
              }
              return fetchedAccount;
            }
            case "seed": {
              const mutable = options.mutable ?? DEFAULTS.WALLET_MUTABLE;
              const authScheme = options.authScheme ?? DEFAULTS.AUTH_SCHEME;
              return await client.importPublicAccountFromSeed(
                options.seed,
                mutable,
                authScheme
              );
            }
          }
        });

        ensureAccountBech32(imported);
        const accounts = accountsAfter ?? (await client.getAccounts());
        setAccounts(accounts);
        setAccount(imported);

        return imported;
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        setError(error);
        throw error;
      } finally {
        setIsImporting(false);
      }
    },
    [client, isReady, runExclusive, setAccounts]
  );

  const reset = useCallback(() => {
    setAccount(null);
    setIsImporting(false);
    setError(null);
  }, []);

  return {
    importAccount,
    account,
    isImporting,
    error,
    reset,
  };
}

function resolveAccountId(accountId: string | AccountIdType): AccountIdType {
  return parseAccountId(accountId);
}

async function resolveAccountFile(
  file: AccountFileType | Uint8Array | ArrayBuffer
): Promise<AccountFileType> {
  if (file instanceof Uint8Array) {
    return AccountFile.deserialize(file);
  }
  if (file instanceof ArrayBuffer) {
    return AccountFile.deserialize(new Uint8Array(file));
  }
  return file;
}

function getAccountFileBytes(
  accountFile: AccountFileType | AccountFileWithAccount,
  original: AccountFileType | Uint8Array | ArrayBuffer
): Uint8Array | null {
  if (original instanceof Uint8Array) {
    return original;
  }
  if (original instanceof ArrayBuffer) {
    return new Uint8Array(original);
  }
  if (typeof accountFile.serialize === "function") {
    return accountFile.serialize();
  }
  return null;
}

function bytesEqual(left: Uint8Array, right: Uint8Array): boolean {
  if (left.length !== right.length) {
    return false;
  }
  for (let i = 0; i < left.length; i += 1) {
    if (left[i] !== right[i]) {
      return false;
    }
  }
  return true;
}
