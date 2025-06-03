import { expect } from "chai";
import { testingPage } from "./mocha.global.setup.mjs";
import {
  createNewFaucet,
  createNewWallet,
  isValidAddress,
  StorageMode,
} from "./webClientTestUtils";

// new_wallet tests
// =======================================================================================================

interface DeployCounterContractResult {
    accountId: string;
}

export const deployCounterContract = async (): Promise<DeployCounterContractResult> => {
    return await testingPage.evaluate(async () => {
        const client = window.client;
        await client.syncState();

        // -------------------------------------------------------------------------
        // STEP 1: Create a basic counter contract
        // -------------------------------------------------------------------------
        const counterMasmCode = `
            use.miden::account
            use.std::sys

            # => []
            export.get_count
                push.0
                # => [index]
                
                exec.account::get_item
                # => [count]
                
                exec.sys::truncate_stack
                # => []
            end

            # => []
            export.increment_count
                push.0
                # => [index]
                
                exec.account::get_item
                # => [count]
                
                push.1 add
                # => [count+1]

                # debug statement with client
                debug.stack

                push.0
                # [index, count+1]
                
                exec.account::set_item
                # => []
                
                push.1 exec.account::incr_nonce
                # => []
                
                exec.sys::truncate_stack
                # => []
            end
        `;

        let assembler = window.TransactionKernel.assembler().withDebugMode(true);

        let storageSlotValue = window.Word.newFromU64s(new BigUint64Array([0n, 0n, 0n, 0n]));
        let storageSlot = window.StorageSlot.fromValue(storageSlotValue);

        let counterComponent = window.AccountComponent.compile(
            counterMasmCode,
            assembler,
            [storageSlot]
        )
        .withSupportsAllTypes();

        const walletSeed = new Uint8Array(32);
        crypto.getRandomValues(walletSeed);

        let anchorBlock = await client.getLatestEpochBlock();

        let accountBuilderResult = new window.AccountBuilder(walletSeed)
            .anchor(anchorBlock)
            .accountType(window.AccountType.RegularAccountImmutableCode)
            .storageMode(window.AccountStorageMode.public())
            .withComponent(counterComponent)
            .build();

        await client.newAccount(
            accountBuilderResult.account,
            accountBuilderResult.seed,
            false
        );

        return {
            accountId: accountBuilderResult.account.id().toString()
        }
    });
}

export const counterContractFpi = async (): Promise<void> => {
    return await testingPage.evaluate(async () => {
        // Step 1
        const counterMasmCode = `\
            use.miden::account
            use.std::sys

            # => []
            export.get_count
                push.0
                # => [index]
                
                exec.account::get_item
                # => [count]
                
                exec.sys::truncate_stack
                # => []
            end

            # => []
            export.increment_count
                push.0
                # => [index]
                
                exec.account::get_item
                # => [count]
                
                push.1 add
                # => [count+1]

                # debug statement with client
                debug.stack

                push.0
                # [index, count+1]
                
                exec.account::set_item
                # => []
                
                push.1 exec.account::incr_nonce
                # => []
                
                exec.sys::truncate_stack
                # => []
            end
        `;

        const countReaderMasmCode = `
            use.miden::account
            use.miden::tx
            use.std::sys

            # => [account_id_prefix, account_id_suffix, get_count_proc_hash]
            export.copy_count
                exec.tx::execute_foreign_procedure
                # => [count]
                
                debug.stack
                # => [count]
                
                push.0
                # [index, count]
                
                exec.account::set_item
                # => []
                
                push.1 exec.account::incr_nonce
                # => []

                exec.sys::truncate_stack
                # => []
            end
        `;

        const readerScriptMasmCode = `
            use.external_contract::count_reader_contract
            use.std::sys

            begin
                # => []
                push.{get_count_proc_hash}

                # => [GET_COUNT_HASH]
                push.{account_id_suffix}

                # => [account_id_suffix]
                push.{account_id_prefix}

                # => []
                push.111 debug.stack drop
                call.count_reader_contract::copy_count

                exec.sys::truncate_stack
            end
        `;
        const client = window.client;
        await client.syncState();

        // -------------------------------------------------------------------------
        // STEP 1: Create the Count Reader Contract
        // -------------------------------------------------------------------------

        let assembler = window.TransactionKernel.assembler().withDebugMode(true);

        const storageSlotValue = window.Word.newFromU64s(new BigUint64Array([0n, 0n, 0n, 0n]))
        const storageSlot = window.StorageSlot.fromValue(storageSlotValue);

        let counterComponent = window.AccountComponent.compile(
            countReaderMasmCode,
            assembler,
            [storageSlot]
        )
        .withSupportsAllTypes();

        const walletSeed = new Uint8Array(32);
        crypto.getRandomValues(walletSeed);

        let anchorBlock = await client.getLatestEpochBlock();

        let accountBuilderResult = new window.AccountBuilder(walletSeed)
            .anchor(anchorBlock)
            .accountType(window.AccountType.RegularAccountImmutableCode)
            .storageMode(window.AccountStorageMode.public())
            .withComponent(counterComponent)
            .build();
        
        await client.newAccount(
            accountBuilderResult.account,
            accountBuilderResult.seed,
            false
        );

        await client.syncState();

        // -------------------------------------------------------------------------
        // STEP 2: Build & Get State of the Counter Contract
        // -------------------------------------------------------------------------

        let deployCounterContractResult = await deployCounterContract();
        let counterContractId = window.AccountId.fromHex(deployCounterContractResult.accountId);

        // -------------------------------------------------------------------------
        // STEP 3: Call the Counter Contract via Foreign Procedure Invocation (FPI)
        // -------------------------------------------------------------------------

        let counterContractComponent = window.AccountComponent.compile(
            counterMasmCode,
            assembler, // This might need a clone
            []
        )
        .withSupportsAllTypes();

        let getCountHash = counterContractComponent.getProcedureHash("get_count");
        let modifiedReaderScriptMasmCode = readerScriptMasmCode
            .replace("{get_count_proc_hash}", getCountHash)
            .replace("{account_id_suffix}", counterContractId.suffix().toString())
            .replace("{account_id_prefix}", counterContractId.prefix().toString());

        let accountComponentLib = window.AssemblerUtils.createAccountComponentLibrary(
            assembler, 
            "external_contract::count_reader_contract", 
            countReaderMasmCode
        );

        let txScript = window.TransactionScript.compile(
            modifiedReaderScriptMasmCode,
            new window.TransactionScriptInputPairArray(),
            assembler.withLibrary(accountComponentLib)
        );

        let foreignAccount = window.ForeignAccount.public(
            counterContractId, 
            new window.AccountStorageRequirements()
        );

        let txRequest = new window.TransactionRequestBuilder()
            .withForeignAccounts([foreignAccount])
            .withCustomScript(txScript)
            .build();

        let txResult = await client.newTransaction(accountBuilderResult.account.id(), txRequest);
        await client.submitTransaction(txResult);

        await client.syncState();
    });
}

describe.only("counter_contract_fpi test", () => {
  it("counter_contract_fpi completes successfully", async () => {
    await expect(counterContractFpi()).to.be.fulfilled;
  });
});
