use alloc::boxed::Box;
use alloc::sync::Arc;

use miden_client::auth::{AuthFalcon512Rpo, AuthSecretKey, RPO_FALCON_SCHEME_ID};
use miden_client::transaction::{
    ProvenTransaction, TransactionExecutorError, TransactionInputs, TransactionProver,
    TransactionProverError, TransactionRequestBuilder,
};
use miden_client::{ClientError, async_trait};
use miden_protocol::account::{AccountBuilder, AccountStorageMode};
use miden_protocol::assembly::diagnostics::miette::GraphicalReportHandler;
use miden_protocol::asset::{Asset, FungibleAsset};
use miden_protocol::note::NoteType;
use miden_protocol::testing::account_id::{
    ACCOUNT_ID_PRIVATE_FUNGIBLE_FAUCET, ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET,
    ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE,
};
use miden_standards::account::wallets::BasicWallet;

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

    let secret_key = AuthSecretKey::new_falcon512_rpo();
    let pub_key = secret_key.public_key();

    let account = AccountBuilder::new(Default::default())
        .with_component(BasicWallet)
        .with_auth_component(AuthFalcon512Rpo::new(pub_key.to_commitment()))
        .with_assets([asset_1, asset_2])
        .build_existing()
        .unwrap();

    keystore.add_key(&secret_key).unwrap();

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

    // Submit transaction
    let _tx_id = Box::pin(client.submit_new_transaction(account.id(), tx_request.clone()))
        .await
        .unwrap();

    // Validate that the request is expected to create two assets in the first note
    let expected_notes = tx_request.expected_output_own_notes();
    assert!(!expected_notes.is_empty());
    assert!(expected_notes[0].assets().num_assets() == 2);

    // Let the client process state changes (mock chain)
    client.sync_state().await.unwrap();
}

#[tokio::test]
async fn transaction_error_reports_source_line() {
    let (mut client, _, keystore) = Box::pin(create_test_client()).await;
    let (wallet, _) = setup_wallet_and_faucet(
        &mut client,
        AccountStorageMode::Private,
        &keystore,
        RPO_FALCON_SCHEME_ID,
    )
    .await
    .unwrap();

    let failing_script = client
        .code_builder()
        .compile_tx_script("begin push.0 push.2 assert_eq end")
        .unwrap();

    let tx_request =
        TransactionRequestBuilder::new().custom_script(failing_script).build().unwrap();

    let err = Box::pin(client.execute_transaction(wallet.id(), tx_request))
        .await
        .expect_err("transaction should fail for assertion");

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

// MOCK PROVERS
// ================================================================================================

/// A prover that always fails with a `TransactionProverError`.
/// Used to test the prover fallback pattern.
struct AlwaysFailingProver;

#[async_trait]
impl TransactionProver for AlwaysFailingProver {
    async fn prove(
        &self,
        _inputs: TransactionInputs,
    ) -> Result<ProvenTransaction, TransactionProverError> {
        Err(TransactionProverError::other("simulated remote prover failure"))
    }
}

// PROVER FALLBACK TESTS
// ================================================================================================

/// Tests the prover fallback pattern: when a remote prover fails, the same transaction
/// request can be retried with a different (local) prover.
#[tokio::test]
async fn prover_fallback_pattern_allows_retry_with_different_prover() {
    let (mut client, _, keystore) = Box::pin(create_test_client()).await;
    let (wallet, faucet) = setup_wallet_and_faucet(
        &mut client,
        AccountStorageMode::Private,
        &keystore,
        RPO_FALCON_SCHEME_ID,
    )
    .await
    .unwrap();

    let fungible_asset = FungibleAsset::new(faucet.id(), 100).unwrap();

    let tx_request = TransactionRequestBuilder::new()
        .build_mint_fungible_asset(fungible_asset, wallet.id(), NoteType::Private, client.rng())
        .unwrap();

    // First attempt with failing prover
    let failing_prover = Arc::new(AlwaysFailingProver);
    let result = Box::pin(client.submit_new_transaction_with_prover(
        faucet.id(),
        tx_request.clone(),
        failing_prover,
    ))
    .await;

    // Verify first attempt fails with TransactionProvingError
    assert!(
        matches!(result, Err(ClientError::TransactionProvingError(_))),
        "expected TransactionProvingError on first attempt"
    );

    // Retry with the client's default prover (which should work)
    let tx_id = Box::pin(client.submit_new_transaction(faucet.id(), tx_request)).await;

    assert!(tx_id.is_ok(), "fallback to default prover should succeed");
}
