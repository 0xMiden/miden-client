import { Page, expect } from "@playwright/test";
import test from "./playwright.global.setup";

export const testStandardFpi = async (page: Page): Promise<void> => {
  return await page.evaluate(async () => {
    console.log("--- INIT TEST ---");
    const client = window.client;
    await client.syncState();

    // BUILD FOREIGN ACCOUNT WITH CUSTOM COMPONENT
    // --------------------------------------------------------------------------

    console.log("--- FELTS ---");
    let felt1 = new window.Felt(15n);
    let felt2 = new window.Felt(15n);
    let felt3 = new window.Felt(15n);
    let felt4 = new window.Felt(15n);
    const MAP_KEY = window.Word.newFromFelts([felt1, felt2, felt3, felt4]);
    const FPI_STORAGE_VALUE = new window.Word(
      new BigUint64Array([9n, 12n, 18n, 30n])
    );

    console.log("--- MAP ---");
    let storageMap = new window.StorageMap();
    storageMap.insert(MAP_KEY, FPI_STORAGE_VALUE);

    const code = `
            export.get_fpi_map_item
                # map key
                push.15.15.15.15
                # item index
                push.0
                exec.::miden::active_account::get_map_item
                swapw dropw
            end
        `;
    let builder = client.createScriptBuilder();
    let getItemComponent = window.AccountComponent.compile(code, builder, [
      window.StorageSlot.map(storageMap),
    ]).withSupportsAllTypes();

    console.log("--- SEED ---");
    const walletSeed = new Uint8Array(32);
    crypto.getRandomValues(walletSeed);

    console.log("--- Secret Key ---");
    let secretKey = window.SecretKey.rpoFalconWithRNG(walletSeed);
    console.log("GENERATED SEED: ", JSON.stringify(walletSeed));
    let authComponent = window.AccountComponent.createAuthComponent(secretKey);

    console.log("--- GET ITEM ACCOUNT BUILDER ---");
    let getItemAccountBuilderResult = new window.AccountBuilder(walletSeed)
      .withAuthComponent(authComponent)
      .withComponent(getItemComponent)
      .storageMode(window.AccountStorageMode.public())
      .build();

    console.log(
      "BUILT ACCOUNT",
      getItemAccountBuilderResult.account.id().toString()
    );
    console.log("BUILT SEED", getItemAccountBuilderResult.seed.toHex());

    console.log("--- PROCEDURE HASH ---");
    let getFpiMapItemProcedureHash =
      getItemComponent.getProcedureHash("get_fpi_map_item");

    // DEPLOY FOREIGN ACCOUNT
    // --------------------------------------------------------------------------

    let foreignAccountId = getItemAccountBuilderResult.account.id();

    console.log("THE FOREIGN ACCOUNT ID: ", foreignAccountId.toString());

    await client.addAccountSecretKeyToWebStore(secretKey);
    await client.newAccount(getItemAccountBuilderResult.account, false);
    await client.syncState();

    let txRequest = new window.TransactionRequestBuilder().build();

    console.log("--- EXECUTE AND APPLY ---");
    let txResult = await window.helpers.executeAndApplyTransaction(
      foreignAccountId,
      txRequest
    );

    console.log("--- EXECUTED ID  ---");
    let txId = txResult.executedTransaction().id();

    await window.helpers.waitForTransaction(txId.toHex());

    // CREATE NATIVE ACCOUNT AND CALL FOREIGN ACCOUNT PROCEDURE VIA FPI
    // --------------------------------------------------------------------------

    let newAccount = await client.newWallet(
      window.AccountStorageMode.public(),
      false
    );

    let txScript = `
            use.miden::tx
            use.miden::account
            begin
                # push the hash of the {} account procedure
                push.{proc_root}

                # push the foreign account id
                push.{account_id_suffix} push.{account_id_prefix}
                # => [foreign_id_prefix, foreign_id_suffix, FOREIGN_PROC_ROOT, storage_item_index]

                exec.tx::execute_foreign_procedure
                push.9.12.18.30 assert_eqw
            end
        `;
    console.log("--- TX SCRIPT ---");
    txScript = txScript
      .replace("{proc_root}", getFpiMapItemProcedureHash)
      .replace("{account_id_suffix}", foreignAccountId.suffix().toString())
      .replace(
        "{account_id_prefix}",
        foreignAccountId.prefix().asInt().toString()
      );

    console.log("--- COMPILED TX SCRIPT ---");
    let compiledTxScript = builder.compileTxScript(txScript);

    await client.syncState();

    await window.helpers.waitForBlocks(2);

    console.log("--- WAITED FOR BLOCKS---");

    let slotAndKeys = new window.SlotAndKeys(1, [MAP_KEY]);
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

    console.log("--- BEFORE TX RESULT 2---");

    let txResult2 = await window.helpers.executeAndApplyTransaction(
      newAccount.id(),
      txRequest2
    );

    console.log("--- TX RESULT 2 APPLIED---");
  });
};

test.describe("fpi test", () => {
  test("runs the standard fpi test successfully", async ({ page }) => {
    await expect(testStandardFpi(page)).resolves.toBeUndefined();
  });
});
