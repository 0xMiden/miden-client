// @ts-nocheck
import { test, expect } from "./test-setup";

// GET_ACCOUNT TESTS
// =======================================================================================================

test.describe("get_account tests", () => {
  test("retrieves an existing account", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
      const newAccount = await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );

      const retrieved = await client.getAccount(newAccount.id());

      return {
        isTruthy: !!retrieved,
        retrievedCommitment: retrieved.to_commitment().toHex(),
        newAccountCommitment: newAccount.to_commitment().toHex(),
      };
    });
    expect(result.isTruthy).toBe(true);
    expect(result.retrievedCommitment).toEqual(result.newAccountCommitment);
  });

  test("returns undefined attempting to retrieve a non-existing account", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk }) => {
      const nonExistingAccountId = sdk.AccountId.fromHex(
        "0x69817bcc6fb9f99027c2245f6979c5"
      );

      const retrieved = await client.getAccount(nonExistingAccountId);

      return { isUndefined: retrieved === undefined };
    });
    expect(result.isUndefined).toBe(true);
  });
});

// GET_ACCOUNTS TESTS
// =======================================================================================================

test.describe("getAccounts tests", () => {
  test("retrieves all existing accounts", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
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

      const accounts = await client.getAccounts();

      const commitmentsOfGetAccountsResult = [];
      for (let i = 0; i < accounts.length; i++) {
        commitmentsOfGetAccountsResult.push(
          accounts[i].to_commitment().toHex()
        );
      }

      return {
        commitmentsOfCreatedAccounts,
        commitmentsOfGetAccountsResult,
        resultLength: accounts.length,
      };
    });

    for (const address of result.commitmentsOfGetAccountsResult) {
      expect(result.commitmentsOfCreatedAccounts.includes(address)).toBe(true);
    }
    expect(result.resultLength).toBe(2);
  });

  test("returns empty array when no accounts exist", async ({ run }) => {
    const result = await run(async ({ client }) => {
      const accounts = await client.getAccounts();
      return { length: accounts.length };
    });
    expect(result.length).toEqual(0);
  });
});

// GET PUBLIC ACCOUNT WITH DETAILS
// =======================================================================================================

test.describe("get public account with details", () => {
  test("assets and storage with too many assets/entries are retrieved", async ({
    run,
  }) => {
    test.skip(true, "requires running node with specific genesis account");
  });
});

// ACCOUNT PUBLIC COMMITMENTS
// =======================================================================================================

test.describe("account public commitments", () => {
  test("properly stores public commitments", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
      const newAccount = await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const accountId = newAccount.id();

      const sk1 = sdk.AuthSecretKey.ecdsaWithRNG(null);
      const sk2 = sdk.AuthSecretKey.rpoFalconWithRNG(null);

      await client.keystore.insert(accountId, sk1);
      await client.keystore.insert(accountId, sk2);

      const commitments = await client.keystore.getCommitments(accountId);

      return { commitmentsLength: commitments.length };
    });
    expect(result.commitmentsLength).toBe(3);
  });

  test("retrieve auth keys with pk commitments and verify signatures", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk }) => {
      const accountId = sdk.AccountId.fromHex(
        "0x69817bcc6fb9f99027c2245f6979c5"
      );

      const sk1 = sdk.AuthSecretKey.ecdsaWithRNG(null);
      const sk2 = sdk.AuthSecretKey.rpoFalconWithRNG(null);
      const sk3 = sdk.AuthSecretKey.rpoFalconWithRNG(null);

      await client.keystore.insert(accountId, sk1);
      await client.keystore.insert(accountId, sk2);
      await client.keystore.insert(accountId, sk3);

      const commitments = await client.keystore.getCommitments(accountId);

      let sk1Retrieved = false;
      let sk2Retrieved = false;
      let sk3Retrieved = false;

      const message = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));
      const signingInputs = sdk.SigningInputs.newBlind(message);

      for (const commitment of commitments) {
        const retrievedSk = await client.keystore.get(commitment);
        const signature = retrievedSk.signData(signingInputs);

        sk1Retrieved =
          sk1Retrieved || sk1.publicKey().verify(message, signature);
        sk2Retrieved =
          sk2Retrieved || sk2.publicKey().verify(message, signature);
        sk3Retrieved =
          sk3Retrieved || sk3.publicKey().verify(message, signature);
      }
      return { allRetrieved: sk1Retrieved && sk2Retrieved && sk3Retrieved };
    });
    expect(result.allRetrieved).toBe(true);
  });

  test("non-registered account id does not have any commitments", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk }) => {
      const accountId = sdk.AccountId.fromHex(
        "0x69817bcc6fb9f99027c2245f6979c5"
      );
      let commitmentsLength;
      try {
        const commitments = await client.keystore.getCommitments(accountId);
        commitmentsLength = commitments.length;
      } catch (e) {
        // On napi (SQLite), querying commitments for an account not in the store
        // throws "account not found" instead of returning an empty array.
        if (e.message?.includes("account not found")) {
          commitmentsLength = 0;
        } else {
          throw e;
        }
      }
      return { commitmentsLength };
    });
    expect(result.commitmentsLength).toBe(0);
  });

  test("can retrieve pk commitment after wallet creation", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
      const account = await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );
      const commitments = await client.keystore.getCommitments(account.id());
      return { commitmentsLength: commitments.length };
    });
    expect(result.commitmentsLength).toBe(1);
  });

  test("separate account ids get their respective pk commitments", async ({
    run,
  }) => {
    const result = await run(async ({ client, sdk }) => {
      const accountId1 = sdk.AccountId.fromHex(
        "0x69817bcc6fb9f99027c2245f6979c5"
      );

      const sk1 = sdk.AuthSecretKey.ecdsaWithRNG(null);
      const sk2 = sdk.AuthSecretKey.rpoFalconWithRNG(null);

      await client.keystore.insert(accountId1, sk1);
      await client.keystore.insert(accountId1, sk2);

      const account1Commitments =
        await client.keystore.getCommitments(accountId1);

      const accountId2 = sdk.AccountId.fromHex(
        "0x79817bcc6fb9f99027c2245f6979ef"
      );

      const sk3 = sdk.AuthSecretKey.rpoFalconWithRNG(null);

      await client.keystore.insert(accountId2, sk3);

      const account2Commitments =
        await client.keystore.getCommitments(accountId2);

      return {
        account1CommitmentsLength: account1Commitments.length,
        account2CommitmentsLength: account2Commitments.length,
      };
    });
    expect(result.account1CommitmentsLength).toBe(2);
    expect(result.account2CommitmentsLength).toBe(1);
  });
});

// GET_ACCOUNT_BY_KEY_COMMITMENT TESTS
// =======================================================================================================

test.describe("getAccountByKeyCommitment tests", () => {
  test("finds wallet by key commitment after creation", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
      const wallet = await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );

      const commitments = await client.keystore.getCommitments(wallet.id());

      const foundAccountId = await client.keystore.getAccountId(commitments[0]);
      const foundAccount = foundAccountId
        ? await client.getAccount(foundAccountId)
        : undefined;

      return {
        foundAccountDefined: foundAccount !== undefined,
        foundAccountId: foundAccount.id().toString(),
        walletId: wallet.id().toString(),
      };
    });
    expect(result.foundAccountDefined).toBe(true);
    expect(result.foundAccountId).toEqual(result.walletId);
  });

  test("returns undefined for non-existent key commitment", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
      const randomSecretKey = sdk.AuthSecretKey.rpoFalconWithRNG(null);
      const randomCommitment = randomSecretKey.publicKey().toCommitment();

      const foundAccountId =
        await client.keystore.getAccountId(randomCommitment);
      const foundAccount = foundAccountId
        ? await client.getAccount(foundAccountId)
        : undefined;

      return { isUndefined: foundAccount === undefined };
    });
    expect(result.isUndefined).toBe(true);
  });

  test("finds correct account among multiple accounts", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
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

      const commitments2 = await client.keystore.getCommitments(wallet2.id());

      const foundAccountId = await client.keystore.getAccountId(
        commitments2[0]
      );
      const foundAccount = foundAccountId
        ? await client.getAccount(foundAccountId)
        : undefined;

      return {
        foundAccountId: foundAccount.id().toString(),
        wallet1Id: wallet1.id().toString(),
        wallet2Id: wallet2.id().toString(),
      };
    });
    expect(result.foundAccountId).toEqual(result.wallet2Id);
    expect(result.foundAccountId).not.toEqual(result.wallet1Id);
  });

  test("finds account by additionally registered key", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
      const wallet = await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );

      const additionalSecretKey = sdk.AuthSecretKey.ecdsaWithRNG(null);
      await client.keystore.insert(wallet.id(), additionalSecretKey);

      const additionalCommitment = additionalSecretKey
        .publicKey()
        .toCommitment();
      const foundAccountId =
        await client.keystore.getAccountId(additionalCommitment);
      const foundAccount = foundAccountId
        ? await client.getAccount(foundAccountId)
        : undefined;

      return {
        foundAccountDefined: foundAccount !== undefined,
        foundAccountId: foundAccount.id().toString(),
        walletId: wallet.id().toString(),
      };
    });
    expect(result.foundAccountDefined).toBe(true);
    expect(result.foundAccountId).toEqual(result.walletId);
  });

  test("finds faucet by key commitment", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
      const faucet = await client.newFaucet(
        sdk.AccountStorageMode.private(),
        false,
        "TST",
        8,
        sdk.u64(10000000),
        sdk.AuthScheme.AuthRpoFalcon512
      );

      const commitments = await client.keystore.getCommitments(faucet.id());

      const foundAccountId = await client.keystore.getAccountId(commitments[0]);
      const foundAccount = foundAccountId
        ? await client.getAccount(foundAccountId)
        : undefined;

      return {
        foundAccountId: foundAccount.id().toString(),
        faucetId: faucet.id().toString(),
        isFaucet: foundAccount.isFaucet(),
      };
    });
    expect(result.foundAccountId).toEqual(result.faucetId);
    expect(result.isFaucet).toBe(true);
  });
});
