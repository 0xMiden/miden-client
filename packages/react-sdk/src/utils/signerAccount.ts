import type { WasmWebClient as WebClient } from "@miden-sdk/miden-sdk";
import type {
  SignerAccountConfig,
  SignerAccountType,
} from "../context/SignerContext";

// SIGNER ACCOUNT INITIALIZATION
// ================================================================================================

/**
 * Maps SignerAccountType string to SDK AccountType enum value.
 */
async function getAccountType(accountType: SignerAccountType) {
  const { AccountType } = await import("@miden-sdk/miden-sdk");

  switch (accountType) {
    case "RegularAccountImmutableCode":
      return AccountType.RegularAccountImmutableCode;
    case "RegularAccountUpdatableCode":
      return AccountType.RegularAccountUpdatableCode;
    case "FungibleFaucet":
      return AccountType.FungibleFaucet;
    case "NonFungibleFaucet":
      return AccountType.NonFungibleFaucet;
    default:
      return AccountType.RegularAccountImmutableCode;
  }
}

/**
 * Checks if the storage mode represents a private account.
 */
function isPrivateStorageMode(
  storageMode: import("@miden-sdk/miden-sdk").AccountStorageMode
): boolean {
  // AccountStorageMode.toString() returns "private", "public", or "network"
  return storageMode.toString() === "private";
}

/**
 * Initializes an account from signer configuration.
 *
 * This function:
 * 1. Syncs the client state
 * 2. Builds an account with the signer's public key commitment as the auth component
 * 3. Attempts to import from chain if public/network storage mode
 * 4. Creates the account locally if it doesn't exist
 *
 * @param client - The WebClient instance
 * @param config - The signer account configuration
 * @returns The account ID as a string
 */
export async function initializeSignerAccount(
  client: WebClient,
  config: SignerAccountConfig
): Promise<string> {
  const { AccountBuilder, AccountComponent, Word } =
    await import("@miden-sdk/miden-sdk");

  // Sync first to get latest state
  await client.syncState();

  // Convert Uint8Array commitment to Word (required by SDK)
  const commitmentWord = Word.deserialize(config.publicKeyCommitment);

  // Build account with auth component from public key commitment
  const seed = config.accountSeed ?? crypto.getRandomValues(new Uint8Array(32));
  const accountType = await getAccountType(config.accountType);

  let builder = new AccountBuilder(seed)
    .withAuthComponent(
      AccountComponent.createAuthComponentFromCommitment(
        commitmentWord,
        1 // ECDSA auth scheme (K256/Keccak)
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
