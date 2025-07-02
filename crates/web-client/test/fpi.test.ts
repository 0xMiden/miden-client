import { expect } from "chai";
import { testingPage } from "./mocha.global.setup.mjs";

export const testStandardFpi = async (): Promise<void> => {
  return await testingPage.evaluate(async () => {
    const client = window.client;
    await client.syncState();

    // BUILD FOREIGN ACCOUNT WITH CUSTOM COMPONENT
    // --------------------------------------------------------------------------

    let felt1 = new window.Felt(15n);
    let felt2 = new window.Felt(15n);
    let felt3 = new window.Felt(15n);
    let felt4 = new window.Felt(15n);
    const MAP_KEY = new window.RpoDigest([felt1, felt2, felt3, felt4]);
    const FPI_STORAGE_VALUE = window.Word.newFromU64s(
      new BigUint64Array([9n, 12n, 18n, 30n])
    );

    let storageMap = new window.StorageMap();
    storageMap.insert(MAP_KEY, FPI_STORAGE_VALUE);

    const code = `
            export.get_fpi_map_item
                # map key
                push.15.15.15.15
                # item index
                push.0
                exec.::miden::account::get_map_item
                swapw dropw
            end
        `;

    let getItemComponent = window.AccountComponent.compile(
      code,
      window.TransactionKernel.assembler(),
      [window.StorageSlot.map(storageMap)]
    ).withSupportsAllTypes();

    const walletSeed = new Uint8Array(32);
    crypto.getRandomValues(walletSeed);

    let secretKey = window.SecretKey.withRng(walletSeed);
    let authComponent = window.AccountComponent.createAuthComponent(secretKey);

    let anchorBlock = await client.getLatestEpochBlock();

    let getItemAccountBuilderResult = new window.AccountBuilder(walletSeed)
      .anchor(anchorBlock)
      .withComponent(authComponent)
      .withComponent(getItemComponent)
      .storageMode(window.AccountStorageMode.public())
      .build();

    let getFpiMapItemProcedureHash =
      getItemComponent.getProcedureHash("get_fpi_map_item");

    // DEPLOY FOREIGN ACCOUNT
    // --------------------------------------------------------------------------

    let foreignAccountId = getItemAccountBuilderResult.account.id();

    await client.addAccountSecretKeyToWebStore(secretKey);
    await client.newAccount(
      getItemAccountBuilderResult.account,
      getItemAccountBuilderResult.seed,
      false
    );
    await client.syncState();

    let deploymentTxScript = window.TransactionScript.compile(
      `
                begin 
                    call.::miden::contracts::auth::basic::auth_tx_rpo_falcon512 
                end
            `,
      new window.TransactionScriptInputPairArray(),
      window.TransactionKernel.assembler()
    );

    let txRequest = new window.TransactionRequestBuilder()
      .withCustomScript(deploymentTxScript)
      .build();

    let txResult = await client.newTransaction(foreignAccountId, txRequest);

    let txId = txResult.executedTransaction().id();

    await client.submitTransaction(txResult);

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
        
                call.::miden::contracts::auth::basic::auth_tx_rpo_falcon512 
            end
        `;
    txScript = txScript
      .replace("{proc_root}", getFpiMapItemProcedureHash)
      .replace("{account_id_suffix}", foreignAccountId.suffix().toString())
      .replace(
        "{account_id_prefix}",
        foreignAccountId.prefix().asInt().toString()
      );

    let compiledTxScript = window.TransactionScript.compile(
      txScript,
      new window.TransactionScriptInputPairArray(),
      window.TransactionKernel.assembler()
    );

    await client.syncState();

    await window.helpers.waitForBlocks(2);

    let slotAndKeys = new window.SlotAndKeys(1, [MAP_KEY]);
    let storageRequirements =
      window.AccountStorageRequirements.fromSlotAndKeysArray([slotAndKeys]);

    let foreignAccount = window.ForeignAccount.public(
      foreignAccountId,
      storageRequirements
    );

    let txRequest2 = new window.TransactionRequestBuilder()
      .withCustomScript(compiledTxScript)
      .withForeignAccounts([foreignAccount])
      .build();

    let txResult2 = await client.newTransaction(newAccount.id(), txRequest2);

    await client.submitTransaction(txResult2);
  });
};

describe("fpi test", () => {
  it("runs the standard fpi test successfully", async () => {
    await expect(testStandardFpi()).to.be.fulfilled;
  }).timeout(50000);
});
