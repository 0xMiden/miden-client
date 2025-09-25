use alloc::boxed::Box;

use miden_client::ClientError;
use miden_client::transaction::{
    TransactionExecutorError,
    TransactionRequestBuilder,
    TransactionResult,
};
use miden_lib::account::auth::AuthRpoFalcon512;
use miden_lib::transaction::TransactionKernel;
use miden_lib::utils::{Deserializable, Serializable};
use miden_objects::Word;
use miden_objects::account::{
    AccountBuilder,
    AccountComponent,
    AccountStorageMode,
    AuthSecretKey,
    StorageMap,
    StorageSlot,
};
use miden_objects::assembly::diagnostics::miette::GraphicalReportHandler;
use miden_objects::asset::{Asset, FungibleAsset};
use miden_objects::crypto::dsa::rpo_falcon512::SecretKey;
use miden_objects::note::NoteType;
use miden_objects::testing::account_id::{
    ACCOUNT_ID_PRIVATE_FUNGIBLE_FAUCET,
    ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET,
    ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE,
};

use super::PaymentNoteDescription;
use crate::tests::{create_test_client, setup_wallet_and_faucet};

#[tokio::test]
async fn transaction_creates_two_notes() {
    let (mut client, _, keystore) = Box::pin(create_test_client()).await;
    let asset_1: Asset =
        FungibleAsset::new(ACCOUNT_ID_PRIVATE_FUNGIBLE_FAUCET.try_into().unwrap(), 123)
            .unwrap()
            .into();
    let asset_2: Asset =
        FungibleAsset::new(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET.try_into().unwrap(), 500)
            .unwrap()
            .into();

    let secret_key = SecretKey::new();
    let pub_key = secret_key.public_key();
    keystore.add_key(&AuthSecretKey::RpoFalcon512(secret_key)).unwrap();

    let wallet_component = AccountComponent::compile(
        "
            export.::miden::contracts::wallets::basic::receive_asset
            export.::miden::contracts::wallets::basic::move_asset_to_note
        ",
        TransactionKernel::assembler(),
        vec![StorageSlot::Value(Word::default()), StorageSlot::Map(StorageMap::default())],
    )
    .unwrap()
    .with_supports_all_types();

    let account = AccountBuilder::new(Default::default())
        .with_component(wallet_component)
        .with_auth_component(AuthRpoFalcon512::new(pub_key))
        .with_assets([asset_1, asset_2])
        .build_existing()
        .unwrap();

    client.add_account(&account, false).await.unwrap();
    client.sync_state().await.unwrap();
    let tx_request = TransactionRequestBuilder::new()
        .build_pay_to_id(
            PaymentNoteDescription::new(
                vec![asset_1, asset_2],
                account.id(),
                ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE.try_into().unwrap(),
            ),
            NoteType::Private,
            client.rng(),
        )
        .unwrap();

    let tx_result = Box::pin(client.new_transaction(account.id(), tx_request)).await.unwrap();
    assert!(
        tx_result
            .created_notes()
            .get_note(0)
            .assets()
            .is_some_and(|assets| assets.num_assets() == 2)
    );
    // Prove and apply transaction
    Box::pin(client.testing_apply_transaction(tx_result.clone())).await.unwrap();

    // Test serialization
    let bytes: std::vec::Vec<u8> = tx_result.to_bytes();
    let decoded = TransactionResult::read_from_bytes(&bytes).unwrap();

    assert_eq!(tx_result, decoded);
}

#[tokio::test]
async fn transaction_error_reports_source_line() {
    let (mut client, _, keystore) = Box::pin(create_test_client()).await;
    let (wallet, _) = setup_wallet_and_faucet(&mut client, AccountStorageMode::Private, &keystore)
        .await
        .unwrap();

    let failing_script = client
        .script_builder()
        .compile_tx_script("begin push.0 push.2 assert_eq end")
        .unwrap();

    let tx_request =
        TransactionRequestBuilder::new().custom_script(failing_script).build().unwrap();

    let err = Box::pin(client.new_transaction(wallet.id(), tx_request)).await.unwrap_err();

    let source_snippet = "push.0 push.2";
    match err {
        ClientError::TransactionExecutorError(
            TransactionExecutorError::TransactionProgramExecutionFailed(exec_err),
        ) => {
            let mut rendered = String::new();
            GraphicalReportHandler::new()
                .render_report(&mut rendered, exec_err.as_ref())
                .unwrap();

            assert!(
                rendered.contains(source_snippet),
                "expected execution error to include script snippet; got:\n{rendered}"
            );
        },
        other => panic!("unexpected error variant: {other:?}"),
    }
}
