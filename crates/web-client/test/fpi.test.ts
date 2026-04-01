// @ts-nocheck
import { test, expect } from "./test-setup";

test.describe("fpi test", () => {
  test("runs the standard fpi test successfully and verifies account proof", async ({
    run,
  }) => {
    test.slow();
    const result = await run(async ({ sdk, helpers }) => {
      const integration = await helpers.createIntegrationClient();
      if (!integration) return { skip: true };
      const { client: intClient } = integration;

      await intClient.syncState();

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
      let builder = await intClient.createCodeBuilder();
      let componentLibrary = builder.buildLibrary(COMPONENT_LIB_PATH, code);
      let getItemComponent = sdk.AccountComponent.fromLibrary(
        componentLibrary,
        [sdk.StorageSlot.map(MAP_SLOT_NAME, storageMap)]
      ).withSupportsAllTypes();

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

      await intClient.keystore.insert(foreignAccountId, secretKey);
      await intClient.newAccount(getItemAccountBuilderResult.account, false);
      await intClient.syncState();

      let txRequest = new sdk.TransactionRequestBuilder().build();

      let txResult = await intClient.executeTransaction(
        foreignAccountId,
        txRequest
      );
      let prover = sdk.TransactionProver.newLocalProver();
      let proven = await intClient.proveTransaction(txResult, prover);
      let height = await intClient.submitProvenTransaction(proven, txResult);
      await intClient.applyTransaction(txResult, height);

      let txId = txResult.executedTransaction().id().toHex();

      // Wait for deploy tx
      let timeWaited = 0;
      while (timeWaited < 10000) {
        await intClient.syncState();
        const uncommitted = await intClient.getTransactions(
          sdk.TransactionFilter.uncommitted()
        );
        const ids = uncommitted.map((tx) => tx.id().toHex());
        if (!ids.includes(txId)) break;
        await new Promise((r) => setTimeout(r, 1000));
        timeWaited += 1000;
      }

      // CREATE NATIVE ACCOUNT AND CALL FOREIGN ACCOUNT PROCEDURE VIA FPI
      // --------------------------------------------------------------------------

      let newAccount = await intClient.newWallet(
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

      await intClient.syncState();

      // Wait for 2 blocks
      const startBlock = await intClient.getSyncHeight();
      while (true) {
        const summary = await intClient.syncState();
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
        .withForeignAccounts(new sdk.ForeignAccountArray([foreignAccount]))
        .build();

      let txResult2 = await intClient.executeTransaction(
        newAccount.id(),
        txRequest2
      );
      prover = sdk.TransactionProver.newLocalProver();
      proven = await intClient.proveTransaction(txResult2, prover);
      height = await intClient.submitProvenTransaction(proven, txResult2);
      await intClient.applyTransaction(txResult2, height);

      const foreignAccountIdStr = foreignAccountId.toString();

      // Test RpcClient.getAccountProof on the deployed public account
      const rpcUrl = helpers.getRpcUrl();
      const endpoint = new sdk.Endpoint(rpcUrl);
      const rpcClient = new sdk.RpcClient(endpoint);

      const accountId = sdk.AccountId.fromHex(foreignAccountIdStr);
      const accountProof = await rpcClient.getAccountProof(accountId);

      return {
        skip: false,
        foreignAccountIdStr,
        proofAccountId: accountProof.accountId().toString(),
        proofBlockNum: accountProof.blockNum(),
        proofCommitmentHex: accountProof.accountCommitment().toHex(),
        hasAccountHeader: !!accountProof.accountHeader(),
        hasAccountCode: !!accountProof.accountCode(),
        numStorageSlots: accountProof.numStorageSlots(),
        headerNonceDefined:
          accountProof.accountHeader()?.nonce().toString() !== undefined,
      };
    });
    if (result.skip) {
      test.skip(true, "requires running node");
      return;
    }
    expect(result.proofAccountId).toEqual(result.foreignAccountIdStr);
    expect(result.proofBlockNum).toBeGreaterThan(0);
    expect(result.proofCommitmentHex).toMatch(/^0x[0-9a-fA-F]+$/);
    expect(result.hasAccountHeader).toBe(true);
    expect(result.hasAccountCode).toBe(true);
    expect(result.numStorageSlots).toBeGreaterThan(0);
    expect(result.headerNonceDefined).toBeDefined();
  });
});
