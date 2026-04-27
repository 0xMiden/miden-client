import type { WasmWebClient as WebClient } from "@miden-sdk/miden-sdk/lazy";
import type {
  SignerAccountConfig,
  SignerAccountType,
} from "../context/SignerContext";
import { parseAccountId } from "./accountParsing";

// SIGNER ACCOUNT INITIALIZATION
// ================================================================================================

/**
 * WASM AccountType enum values (from wasm-bindgen).
 * We define these directly because the simplified API's AccountType const
 * shadows the WASM enum with string aliases for some variants.
 */
const WASM_ACCOUNT_TYPE: Record<SignerAccountType, number> = {
  FungibleFaucet: 0,
  NonFungibleFaucet: 1,
  RegularAccountImmutableCode: 2,
  RegularAccountUpdatableCode: 3,
};

/**
 * Maps SignerAccountType string to the WASM AccountType enum numeric value.
 */
function getAccountType(accountType: SignerAccountType): number {
  return (
    WASM_ACCOUNT_TYPE[accountType] ??
    WASM_ACCOUNT_TYPE.RegularAccountImmutableCode
  );
}

/**
 * Checks if the storage mode represents a private account.
 */
function isPrivateStorageMode(
  storageMode: import("@miden-sdk/miden-sdk/lazy").AccountStorageMode
): boolean {
  // AccountStorageMode.toString() returns "private", "public", or "network"
  return storageMode.toString() === "private";
}

/**
 * Initializes an account from signer configuration.
 *
 * Resolution order when `importAccountId` is set:
 * 1. **Public storage:** `importAccountById` (freshest chain state). On
 *    "not found on the network", fall through to step 2.
 * 2. If `accountFileBytes` is provided, `importAccountFile` (works for
 *    public-unfunded AND private accounts — chain doesn't carry state for
 *    either). The wallet is responsible for serializing the AccountFile
 *    such that its `account.id()` matches `importAccountId`.
 * 3. Throw — neither chain nor wallet had the account.
 *
 * When `importAccountId` is unset, falls through to the legacy build-from-
 * publicKeyCommitment path used by signers that don't share an externally-
 * created account ID up-front.
 *
 * @param client - The WebClient instance
 * @param config - The signer account configuration
 * @returns The account ID as a string
 */
export async function initializeSignerAccount(
  client: WebClient,
  config: SignerAccountConfig
): Promise<string> {
  const {
    AccountBuilder,
    AccountComponent,
    AccountFile,
    AuthScheme,
    Word,
    resolveAuthScheme,
  } = await import("@miden-sdk/miden-sdk/lazy");

  // Sync first to get latest state
  await client.syncState();

  // Fast path: import existing account by ID instead of rebuilding from scratch.
  if (config.importAccountId) {
    const accountId = parseAccountId(config.importAccountId);
    const isPrivate = isPrivateStorageMode(config.storageMode);

    // Public-storage accounts: try chain first (freshest state). For private
    // accounts, skip chain entirely — state lives off-chain so an
    // importAccountById call can only fail or be misleading.
    if (!isPrivate) {
      try {
        await client.importAccountById(accountId);
        await client.syncState();
        return config.importAccountId;
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        if (msg.includes("already being tracked")) {
          await client.syncState();
          return config.importAccountId;
        }
        // "not found on the network" → fall through to AccountFile path
      }
    }

    // AccountFile fallback: the wallet shipped a serialized snapshot. Works
    // for public-unfunded (chain has nothing yet) and private (chain never
    // has it) accounts.
    if (config.accountFileBytes) {
      const file = AccountFile.deserialize(config.accountFileBytes);
      try {
        await client.importAccountFile(file);
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        if (!msg.includes("already being tracked")) {
          throw e;
        }
      }
      await client.syncState();
      return config.importAccountId;
    }

    throw new Error(
      `Account ${config.importAccountId} not found on the network and the signer ` +
        `did not provide accountFileBytes. For unfunded public accounts or private ` +
        `accounts, the signer must populate accountConfig.accountFileBytes.`
    );
  }

  // Convert Uint8Array commitment to Word (required by SDK)
  const commitmentWord = Word.deserialize(config.publicKeyCommitment);

  // Build account with auth component from public key commitment
  const seed = config.accountSeed ?? crypto.getRandomValues(new Uint8Array(32));
  const accountType = getAccountType(config.accountType);

  let builder = new AccountBuilder(seed)
    .withAuthComponent(
      AccountComponent.createAuthComponentFromCommitment(
        commitmentWord,
        resolveAuthScheme(AuthScheme.ECDSA)
      )
    )
    // eslint-disable-next-line @typescript-eslint/no-explicit-any -- SDK type mismatch between JS wrapper AccountType and WASM enum AccountType
    .accountType(accountType as any)
    .storageMode(config.storageMode)
    .withBasicWalletComponent();

  // Add any custom components (e.g. from compiled .masp packages)
  if (config.customComponents?.length) {
    for (const component of config.customComponents) {
      if (
        component == null ||
        typeof (component as any).getProcedures !== "function"
      ) {
        throw new Error(
          "Each entry in customComponents must be an AccountComponent instance created via " +
            "AccountComponent.compile(), AccountComponent.fromPackage(), or AccountComponent.fromLibrary()."
        );
      }
      builder = builder.withComponent(component);
    }
  }

  const buildResult = builder.build();

  const account = buildResult.account;
  const accountId = account.id();

  // For public/network accounts, try to import from chain first
  if (!isPrivateStorageMode(config.storageMode)) {
    try {
      await client.importAccountById(accountId);
      // Account imported successfully from chain
      await client.syncState();
      return accountId.toString();
    } catch {
      // Account doesn't exist on-chain yet, will create locally
    }
  }

  // Check if account already exists locally
  try {
    const existing = await client.getAccount(accountId);
    if (existing) {
      await client.syncState();
      return accountId.toString();
    }
  } catch {
    // Account doesn't exist locally
  }

  // Create account locally
  await client.newAccount(account, false);
  await client.syncState();

  return accountId.toString();
}
