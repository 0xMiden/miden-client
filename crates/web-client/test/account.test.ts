// @ts-nocheck
import { test, expect } from "./test-setup";

// GET_ACCOUNT TESTS
// =======================================================================================================

test.describe("get_account tests", () => {
  test("retrieves an existing account", async ({ client, sdk }) => {
    const newAccount = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512
    );

    const result = await client.getAccount(newAccount.id());

    expect(result).toBeTruthy();
    expect(result.to_commitment().toHex()).toEqual(
      newAccount.to_commitment().toHex()
    );
  });

  test("returns undefined attempting to retrieve a non-existing account", async ({
    client,
    sdk,
  }) => {
    const nonExistingAccountId = sdk.TestUtils.createMockAccountId();

    const result = await client.getAccount(nonExistingAccountId);

    expect(result).toBeUndefined();
  });
});

// GET_ACCOUNTS TESTS
// =======================================================================================================

test.describe("getAccounts tests", () => {
  test("retrieves all existing accounts", async ({ client, sdk }) => {
    const newAccount1 = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512
    );
    const newAccount2 = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512
    );
    const commitmentsOfCreatedAccounts = [
      newAccount1.to_commitment().toHex(),
      newAccount2.to_commitment().toHex(),
    ];

    const result = await client.getAccounts();

    const commitmentsOfGetAccountsResult = [];
    for (let i = 0; i < result.length; i++) {
      commitmentsOfGetAccountsResult.push(result[i].to_commitment().toHex());
    }

    for (const address of commitmentsOfGetAccountsResult) {
      expect(commitmentsOfCreatedAccounts.includes(address)).toBe(true);
    }
    expect(result.length).toBe(2);
  });

  test("returns empty array when no accounts exist", async ({ client }) => {
    const result = await client.getAccounts();

    expect(result.length).toEqual(0);
  });
});

// GET PUBLIC ACCOUNT WITH DETAILS
// =======================================================================================================

test.describe("get public account with details", () => {
  test("assets and storage with too many assets/entries are retrieved", async ({
    client,
    sdk,
  }) => {
    test.skip(true, "requires running node with specific genesis account");
  });
});

// ACCOUNT PUBLIC COMMITMENTS
// =======================================================================================================

test.describe("account public commitments", () => {
  test("properly stores public commitments", async ({ client, sdk }) => {
    const newAccount = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512
    );
    const accountId = newAccount.id();

    const sk1 = sdk.AuthSecretKey.ecdsaWithRNG(null);
    const sk2 = sdk.AuthSecretKey.rpoFalconWithRNG(null);

    await client.addAccountSecretKeyToWebStore(accountId, sk1);
    await client.addAccountSecretKeyToWebStore(accountId, sk2);

    const commitments =
      await client.getPublicKeyCommitmentsOfAccount(accountId);

    expect(commitments.length).toBe(3);
  });

  test("retrieve auth keys with pk commitments and verify signatures", async ({
    client,
    sdk,
  }) => {
    const accountId = sdk.AccountId.fromHex("0x69817bcc6fb9f99027c2245f6979c5");

    const sk1 = sdk.AuthSecretKey.ecdsaWithRNG(null);
    const sk2 = sdk.AuthSecretKey.rpoFalconWithRNG(null);
    const sk3 = sdk.AuthSecretKey.rpoFalconWithRNG(null);

    await client.addAccountSecretKeyToWebStore(accountId, sk1);
    await client.addAccountSecretKeyToWebStore(accountId, sk2);
    await client.addAccountSecretKeyToWebStore(accountId, sk3);

    const commitments =
      await client.getPublicKeyCommitmentsOfAccount(accountId);

    let sk1Retrieved = false;
    let sk2Retrieved = false;
    let sk3Retrieved = false;

    const message = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));
    const signingInputs = sdk.SigningInputs.newBlind(message);

    for (const commitment of commitments) {
      const retrievedSk =
        await client.getAccountAuthByPubKeyCommitment(commitment);
      const signature = retrievedSk.signData(signingInputs);

      sk1Retrieved = sk1Retrieved || sk1.publicKey().verify(message, signature);
      sk2Retrieved = sk2Retrieved || sk2.publicKey().verify(message, signature);
      sk3Retrieved = sk3Retrieved || sk3.publicKey().verify(message, signature);
    }
    expect(sk1Retrieved && sk2Retrieved && sk3Retrieved).toBe(true);
  });

  test("non-registered account id does not have any commitments", async ({
    client,
    sdk,
  }) => {
    const accountId = sdk.AccountId.fromHex("0x69817bcc6fb9f99027c2245f6979c5");
    const commitments =
      await client.getPublicKeyCommitmentsOfAccount(accountId);
    expect(commitments.length).toBe(0);
  });

  test("can retrieve pk commitment after wallet creation", async ({
    client,
    sdk,
  }) => {
    const account = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512
    );
    const commitments = await client.getPublicKeyCommitmentsOfAccount(
      account.id()
    );
    expect(commitments.length).toBe(1);
  });

  test("separate account ids get their respective pk commitments", async ({
    client,
    sdk,
  }) => {
    const accountId1 = sdk.AccountId.fromHex(
      "0x69817bcc6fb9f99027c2245f6979c5"
    );

    const sk1 = sdk.AuthSecretKey.ecdsaWithRNG(null);
    const sk2 = sdk.AuthSecretKey.rpoFalconWithRNG(null);

    await client.addAccountSecretKeyToWebStore(accountId1, sk1);
    await client.addAccountSecretKeyToWebStore(accountId1, sk2);

    const account1Commitments =
      await client.getPublicKeyCommitmentsOfAccount(accountId1);

    const accountId2 = sdk.AccountId.fromHex(
      "0x79817bcc6fb9f99027c2245f6979ef"
    );

    const sk3 = sdk.AuthSecretKey.rpoFalconWithRNG(null);

    await client.addAccountSecretKeyToWebStore(accountId2, sk3);

    const account2Commitments =
      await client.getPublicKeyCommitmentsOfAccount(accountId2);

    expect(account1Commitments.length).toBe(2);
    expect(account2Commitments.length).toBe(1);
  });
});

// GET_ACCOUNT_BY_KEY_COMMITMENT TESTS
// =======================================================================================================

test.describe("getAccountByKeyCommitment tests", () => {
  test("finds wallet by key commitment after creation", async ({
    client,
    sdk,
  }) => {
    const wallet = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512
    );

    const commitments = await client.getPublicKeyCommitmentsOfAccount(
      wallet.id()
    );

    const foundAccount = await client.getAccountByKeyCommitment(commitments[0]);

    expect(foundAccount).toBeDefined();
    expect(foundAccount.id().toString()).toEqual(wallet.id().toString());
  });

  test("returns undefined for non-existent key commitment", async ({
    client,
    sdk,
  }) => {
    const randomSecretKey = sdk.AuthSecretKey.rpoFalconWithRNG(null);
    const randomCommitment = randomSecretKey.publicKey().toCommitment();

    const foundAccount =
      await client.getAccountByKeyCommitment(randomCommitment);

    expect(foundAccount).toBeUndefined();
  });

  test("finds correct account among multiple accounts", async ({
    client,
    sdk,
  }) => {
    const wallet1 = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512
    );
    const wallet2 = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512
    );

    const commitments2 = await client.getPublicKeyCommitmentsOfAccount(
      wallet2.id()
    );

    const foundAccount = await client.getAccountByKeyCommitment(
      commitments2[0]
    );

    expect(foundAccount.id().toString()).toEqual(wallet2.id().toString());
    expect(foundAccount.id().toString()).not.toEqual(wallet1.id().toString());
  });

  test("finds account by additionally registered key", async ({
    client,
    sdk,
  }) => {
    const wallet = await client.newWallet(
      sdk.AccountStorageMode.private(),
      true,
      sdk.AuthScheme.AuthRpoFalcon512
    );

    const additionalSecretKey = sdk.AuthSecretKey.ecdsaWithRNG(null);
    await client.addAccountSecretKeyToWebStore(
      wallet.id(),
      additionalSecretKey
    );

    const additionalCommitment = additionalSecretKey.publicKey().toCommitment();
    const foundAccount =
      await client.getAccountByKeyCommitment(additionalCommitment);

    expect(foundAccount).toBeDefined();
    expect(foundAccount.id().toString()).toEqual(wallet.id().toString());
  });

  test("finds faucet by key commitment", async ({ client, sdk }) => {
    const faucet = await client.newFaucet(
      sdk.AccountStorageMode.private(),
      false,
      "TST",
      8,
      sdk.u64(10000000),
      sdk.AuthScheme.AuthRpoFalcon512
    );

    const commitments = await client.getPublicKeyCommitmentsOfAccount(
      faucet.id()
    );

    const foundAccount = await client.getAccountByKeyCommitment(commitments[0]);

    expect(foundAccount.id().toString()).toEqual(faucet.id().toString());
    expect(foundAccount.isFaucet()).toBe(true);
  });
});
