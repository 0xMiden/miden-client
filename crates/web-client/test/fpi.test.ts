import { Page, expect } from "@playwright/test";
import test from "./playwright.global.setup";

export const testStandardFpi = async (page: Page): Promise<string> => {
  return await page.evaluate(async () => {
    const client = window.client;
    await client.syncState();

    const MAP_SLOT_NAME = "miden::testing::fpi::map_slot";
    const COMPONENT_LIB_PATH = "miden::testing::fpi_component";

    // BUILD FOREIGN ACCOUNT WITH CUSTOM COMPONENT
    // --------------------------------------------------------------------------

    let felt1 = new window.Felt(15n);
    let felt2 = new window.Felt(15n);
    let felt3 = new window.Felt(15n);
    let felt4 = new window.Felt(15n);
    const MAP_KEY = window.Word.newFromFelts([felt1, felt2, felt3, felt4]);
    const FPI_STORAGE_VALUE = new window.Word(
      new BigUint64Array([9n, 12n, 18n, 30n])
    );

    let storageMap = new window.StorageMap();
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
    let builder = client.createCodeBuilder();
    let componentLibrary = builder.buildLibrary(COMPONENT_LIB_PATH, code);
    let getItemComponent = window.AccountComponent.fromLibrary(
      componentLibrary,
      [window.StorageSlot.map(MAP_SLOT_NAME, storageMap)]
    ).withSupportsAllTypes();

    const walletSeed = new Uint8Array(32);
    crypto.getRandomValues(walletSeed);

    let secretKey = window.AuthSecretKey.rpoFalconWithRNG(walletSeed);

    let authComponent =
      window.AccountComponent.createAuthComponentFromSecretKey(secretKey);

    let getItemAccountBuilderResult = new window.AccountBuilder(walletSeed)
      .withAuthComponent(authComponent)
      .withComponent(getItemComponent)
      .storageMode(window.AccountStorageMode.public())
      .build();

    builder.linkDynamicLibrary(componentLibrary);

    // DEPLOY FOREIGN ACCOUNT
    // --------------------------------------------------------------------------

    let foreignAccountId = getItemAccountBuilderResult.account.id();

    await client.addAccountSecretKeyToWebStore(foreignAccountId, secretKey);
    await client.newAccount(getItemAccountBuilderResult.account, false);
    await client.syncState();

    let txRequest = new window.TransactionRequestBuilder().build();

    let txResult = await window.helpers.executeAndApplyTransaction(
      foreignAccountId,
      txRequest
    );

    let txId = txResult.executedTransaction().id();

    await window.helpers.waitForTransaction(txId.toHex());

    // CREATE NATIVE ACCOUNT AND CALL FOREIGN ACCOUNT PROCEDURE VIA FPI
    // --------------------------------------------------------------------------

    let newAccount = await client.newWallet(
      window.AccountStorageMode.public(),
      false,
      0
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
      .replace(
        "{account_id_prefix}",
        foreignAccountId.prefix().asInt().toString()
      );

    let compiledTxScript = builder.compileTxScript(txScript);

    await client.syncState();

    await window.helpers.waitForBlocks(2);

    let slotAndKeys = new window.SlotAndKeys(MAP_SLOT_NAME, [MAP_KEY]);
    let storageRequirements =
      window.AccountStorageRequirements.fromSlotAndKeysArray([slotAndKeys]);

    let foreignAccount = window.ForeignAccount.public(
      foreignAccountId,
      storageRequirements
    );

    let txRequest2 = new window.TransactionRequestBuilder()
      .withCustomScript(compiledTxScript)
      .withForeignAccounts(
        new window.MidenArrays.ForeignAccountArray([foreignAccount])
      )
      .build();

    let txResult2 = await window.helpers.executeAndApplyTransaction(
      newAccount.id(),
      txRequest2
    );

    return foreignAccountId.toString();
  });
};

test.describe("fpi test", () => {
  test("runs the standard fpi test successfully and verifies account proof", async ({
    page,
  }) => {
    const foreignAccountId = await testStandardFpi(page);

    // Test RpcClient.getAccountProof on the deployed public account
    const proofResult = await page.evaluate(
      async (_foreignAccountId: string) => {
        const endpoint = new window.Endpoint(window.rpcUrl);
        const rpcClient = new window.RpcClient(endpoint);

        const accountId = window.AccountId.fromHex(_foreignAccountId);
        const accountProof = await rpcClient.getAccountProof(accountId);

        return {
          accountId: accountProof.accountId().toString(),
          blockNum: accountProof.blockNum(),
          accountCommitment: accountProof.accountCommitment().toHex(),
          hasAccountHeader: !!accountProof.accountHeader(),
          hasAccountCode: !!accountProof.accountCode(),
          numStorageSlots: accountProof.numStorageSlots(),
          nonce: accountProof.accountHeader()?.nonce().toString(),
        };
      },
      foreignAccountId
    );

    expect(proofResult.accountId).toEqual(foreignAccountId);
    expect(proofResult.blockNum).toBeGreaterThan(0);
    expect(proofResult.accountCommitment).toMatch(/^0x[0-9a-fA-F]+$/);
    expect(proofResult.hasAccountHeader).toBe(true);
    expect(proofResult.hasAccountCode).toBe(true);
    expect(proofResult.numStorageSlots).toBeGreaterThan(0);
    expect(proofResult.nonce).toBeDefined();
  });
});
