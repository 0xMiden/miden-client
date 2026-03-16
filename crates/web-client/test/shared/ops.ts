/**
 * Shared SDK operations. Each function takes a client and sdk module,
 * calls the SDK, and returns a plain object. Platform-agnostic.
 */

export async function createNewWallet(
  client: any,
  sdk: any,
  params: { storageMode: string; mutable: boolean }
) {
  const mode =
    params.storageMode === "public"
      ? sdk.AccountStorageMode.public()
      : sdk.AccountStorageMode.private();

  const authScheme = sdk.AuthScheme.AuthRpoFalcon512;
  const wallet = await client.newWallet(mode, params.mutable, authScheme);

  return {
    id: wallet.id().toString(),
    nonce: wallet.nonce().toString(),
    vaultCommitment: wallet.vault().root().toHex(),
    storageCommitment: wallet.storage().commitment().toHex(),
    codeCommitment: wallet.code().commitment().toHex(),
    isFaucet: wallet.isFaucet(),
    isRegularAccount: wallet.isRegularAccount(),
    isUpdatable: wallet.isUpdatable(),
    isPublic: wallet.isPublic(),
    isPrivate: wallet.isPrivate(),
    isNetwork: wallet.isNetwork(),
    isIdPublic: wallet.id().isPublic(),
    isIdPrivate: wallet.id().isPrivate(),
    isIdNetwork: wallet.id().isNetwork(),
    isNew: wallet.isNew(),
  };
}

export async function createNewFaucet(
  client: any,
  sdk: any,
  params: {
    storageMode: string;
    nonFungible: boolean;
    tokenSymbol: string;
    decimals: number;
    maxSupply: number;
  }
) {
  const mode =
    params.storageMode === "public"
      ? sdk.AccountStorageMode.public()
      : sdk.AccountStorageMode.private();

  const authScheme = sdk.AuthScheme.AuthRpoFalcon512;
  const faucet = await client.newFaucet(
    mode,
    params.nonFungible,
    params.tokenSymbol,
    params.decimals,
    params.maxSupply,
    authScheme
  );

  return {
    id: faucet.id().toString(),
    nonce: faucet.nonce().toString(),
    vaultCommitment: faucet.vault().root().toHex(),
    storageCommitment: faucet.storage().commitment().toHex(),
    codeCommitment: faucet.code().commitment().toHex(),
    isFaucet: faucet.isFaucet(),
    isRegularAccount: faucet.isRegularAccount(),
    isUpdatable: faucet.isUpdatable(),
    isPublic: faucet.isPublic(),
    isPrivate: faucet.isPrivate(),
    isNetwork: faucet.isNetwork(),
    isIdPublic: faucet.id().isPublic(),
    isIdPrivate: faucet.id().isPrivate(),
    isIdNetwork: faucet.id().isNetwork(),
    isNew: faucet.isNew(),
  };
}

export async function mockChainMintAndConsume(client: any, sdk: any) {
  await client.syncStateImpl();

  const wallet = await client.newWallet(
    sdk.AccountStorageMode.private(),
    true,
    sdk.AuthScheme.AuthRpoFalcon512
  );
  const faucet = await client.newFaucet(
    sdk.AccountStorageMode.private(),
    false,
    "DAG",
    8,
    10000000,
    sdk.AuthScheme.AuthRpoFalcon512
  );

  const mintRequest = await client.newMintTransactionRequest(
    wallet.id(),
    faucet.id(),
    sdk.NoteType.Public,
    1000
  );
  const mintTxId = await client.submitNewTransaction(faucet.id(), mintRequest);
  await client.proveBlock();
  await client.syncStateImpl();

  const [mintRecord] = await client.getTransactions(
    sdk.TransactionFilter.ids([mintTxId])
  );
  const mintedNoteId = mintRecord.outputNotes().notes()[0].id().toString();
  const noteRecord = await client.getInputNote(mintedNoteId);
  const note = noteRecord.toNote();

  const consumeRequest = client.newConsumeTransactionRequest([note]);
  await client.submitNewTransaction(wallet.id(), consumeRequest);
  await client.proveBlock();
  await client.syncStateImpl();

  const account = await client.getAccount(wallet.id());
  return account.vault().getBalance(faucet.id()).toString();
}
