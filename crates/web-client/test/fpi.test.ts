// @ts-nocheck
import {
  test,
  expect,
  executeAndApplyTransaction,
  waitForTransaction,
} from "./test-setup";
import { createIntegrationClient } from "./test-helpers";
import { getRpcUrl } from "./playwright.global.setup";

test.describe("fpi test", () => {
  test("runs the standard fpi test successfully and verifies account proof", async ({
    sdk,
  }) => {
    test.slow();
    const result = await createIntegrationClient();
    test.skip(!result, "requires running node");
    const { client } = result;

    await client.syncState();

    const MAP_SLOT_NAME = "miden::testing::fpi::map_slot";
    const COMPONENT_LIB_PATH = "miden::testing::fpi_component";

    // BUILD FOREIGN ACCOUNT WITH CUSTOM COMPONENT
    // --------------------------------------------------------------------------

    let felt1 = new sdk.Felt(sdk.u64(15));
    let felt2 = new sdk.Felt(sdk.u64(15));
    let felt3 = new sdk.Felt(sdk.u64(15));
    let felt4 = new sdk.Felt(sdk.u64(15));
    const MAP_KEY = sdk.Word.newFromFelts([felt1, felt2, felt3, felt4]);
    const FPI_STORAGE_VALUE = new sdk.Word(sdk.u64Array([9, 12, 18, 30]));

    let storageMap = new sdk.StorageMap();
    storageMap.insert(MAP_KEY, FPI_STORAGE_VALUE);

    const code = `
            use miden::core::word

            const MAP_SLOT = word("${MAP_SLOT_NAME}")

            pub proc get_fpi_map_item
                # map key
                push.15.15.15.15
                push.MAP_SLOT[0..2]
                exec.::miden::protocol::active_account::get_map_item
                swapw dropw
            end
        `;
    let builder = await client.createCodeBuilder();
    let componentLibrary = builder.buildLibrary(COMPONENT_LIB_PATH, code);
    let getItemComponent = sdk.AccountComponent.fromLibrary(componentLibrary, [
      sdk.StorageSlot.map(MAP_SLOT_NAME, storageMap),
    ]).withSupportsAllTypes();

    const walletSeed = new Uint8Array(32);
    crypto.getRandomValues(walletSeed);

    let secretKey = sdk.AuthSecretKey.rpoFalconWithRNG(walletSeed);

    let authComponent =
      sdk.AccountComponent.createAuthComponentFromSecretKey(secretKey);

    let getItemAccountBuilderResult = new sdk.AccountBuilder(walletSeed)
      .withAuthComponent(authComponent)
      .withComponent(getItemComponent)
      .storageMode(sdk.AccountStorageMode.public())
      .build();

    builder.linkDynamicLibrary(componentLibrary);

    // DEPLOY FOREIGN ACCOUNT
    // --------------------------------------------------------------------------

    let foreignAccountId = getItemAccountBuilderResult.account.id();

    await client.addAccountSecretKeyToWebStore(foreignAccountId, secretKey);
    await client.newAccount(getItemAccountBuilderResult.account, false);
    await client.syncState();

    let txRequest = new sdk.TransactionRequestBuilder().build();

    let txResult = await executeAndApplyTransaction(
      client,
      sdk,
      foreignAccountId,
      txRequest
    );

    let txId = txResult.executedTransaction().id();

    await waitForTransaction(client, sdk, txId.toHex());

    // CREATE NATIVE ACCOUNT AND CALL FOREIGN ACCOUNT PROCEDURE VIA FPI
    // --------------------------------------------------------------------------

    let newAccount = await client.newWallet(
      sdk.AccountStorageMode.public(),
      false,
      sdk.AuthScheme.AuthRpoFalcon512
    );

    let txScript = `
            use miden::protocol::tx
            begin
                # push the hash of the component procedure
                procref.::miden::testing::fpi_component::get_fpi_map_item

                # push the foreign account id
                push.{account_id_suffix} push.{account_id_prefix}
                # => [foreign_id_prefix, foreign_id_suffix, FOREIGN_PROC_ROOT, storage_item_index]

                exec.tx::execute_foreign_procedure
                push.9.12.18.30 assert_eqw
            end
        `;

    txScript = txScript
      .replace("{account_id_suffix}", foreignAccountId.suffix().toString())
      .replace("{account_id_prefix}", foreignAccountId.prefix().toString());

    let compiledTxScript = builder.compileTxScript(txScript);

    await client.syncState();

    // Wait for 2 blocks
    const startBlock = await client.getSyncHeight();
    while (true) {
      const summary = await client.syncState();
      if (summary.blockNum() >= startBlock + 2) break;
      await new Promise((r) => setTimeout(r, 1000));
    }

    let slotAndKeys = new sdk.SlotAndKeys(MAP_SLOT_NAME, [MAP_KEY]);
    let storageRequirements =
      sdk.AccountStorageRequirements.fromSlotAndKeysArray([slotAndKeys]);

    let foreignAccount = sdk.ForeignAccount.public(
      foreignAccountId,
      storageRequirements
    );

    let txRequest2 = new sdk.TransactionRequestBuilder()
      .withCustomScript(compiledTxScript)
      .withForeignAccounts([foreignAccount])
      .build();

    let txResult2 = await executeAndApplyTransaction(
      client,
      sdk,
      newAccount.id(),
      txRequest2
    );

    const foreignAccountIdStr = foreignAccountId.toString();

    // Test RpcClient.getAccountProof on the deployed public account
    const rpcUrl = getRpcUrl();
    const endpoint = new sdk.Endpoint(rpcUrl);
    const rpcClient = new sdk.RpcClient(endpoint);

    const accountId = sdk.AccountId.fromHex(foreignAccountIdStr);
    const accountProof = await rpcClient.getAccountProof(accountId);

    expect(accountProof.accountId().toString()).toEqual(foreignAccountIdStr);
    expect(accountProof.blockNum()).toBeGreaterThan(0);
    expect(accountProof.accountCommitment().toHex()).toMatch(
      /^0x[0-9a-fA-F]+$/
    );
    expect(!!accountProof.accountHeader()).toBe(true);
    expect(!!accountProof.accountCode()).toBe(true);
    expect(accountProof.numStorageSlots()).toBeGreaterThan(0);
    expect(accountProof.accountHeader()?.nonce().toString()).toBeDefined();
  });
});
