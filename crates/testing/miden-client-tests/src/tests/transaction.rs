use alloc::boxed::Box;

use miden_client::transaction::TransactionRequestBuilder;
use miden_lib::account::auth::AuthRpoFalcon512;
use miden_lib::transaction::TransactionKernel;
use miden_objects::Word;
use miden_objects::account::{
    AccountBuilder,
    AccountComponent,
    AuthSecretKey,
    StorageMap,
    StorageSlot,
};
use miden_objects::asset::{Asset, FungibleAsset};
use miden_objects::crypto::dsa::rpo_falcon512::SecretKey;
use miden_objects::note::NoteType;
use miden_objects::testing::account_id::{
    ACCOUNT_ID_PRIVATE_FUNGIBLE_FAUCET,
    ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET,
    ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE,
};

use super::PaymentNoteDescription;
use crate::tests::create_test_client;

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

    client.add_account(&account, None, false).await.unwrap();
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

    // Submit transaction
    let _tx_id = Box::pin(client.new_transaction(account.id(), tx_request.clone()))
        .await
        .unwrap();

    // Validate that the request is expected to create two assets in the first note
    let expected_notes = tx_request.expected_output_own_notes();
    assert!(!expected_notes.is_empty());
    assert!(expected_notes[0].assets().num_assets() == 2);

    // Let the client process state changes (mock chain)
    client.sync_state().await.unwrap();
}
