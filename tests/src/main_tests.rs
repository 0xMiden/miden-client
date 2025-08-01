mod custom_transactions_tests;
mod fpi_tests;
mod network_transaction_tests;
mod onchain_tests;
mod swap_transactions_tests;

#[tokio::test]
async fn client_builder_initializes_client_with_endpoint() {
    miden_client_integration_tests::tests::client::client_builder_initializes_client_with_endpoint(
        Default::default(),
    )
    .await
}

#[tokio::test]
async fn multiple_tx_on_same_block() {
    miden_client_integration_tests::tests::client::multiple_tx_on_same_block(Default::default())
        .await
}

#[tokio::test]
async fn import_expected_notes() {
    miden_client_integration_tests::tests::client::import_expected_notes(Default::default()).await
}

#[tokio::test]
async fn import_expected_note_uncommitted() {
    miden_client_integration_tests::tests::client::import_expected_note_uncommitted(
        Default::default(),
    )
    .await
}

#[tokio::test]
async fn import_expected_notes_from_the_past_as_committed() {
    miden_client_integration_tests::tests::client::import_expected_notes_from_the_past_as_committed(
        Default::default(),
    )
    .await
}

#[tokio::test]
async fn get_account_update() {
    miden_client_integration_tests::tests::client::get_account_update(Default::default()).await
}

#[tokio::test]
async fn sync_detail_values() {
    miden_client_integration_tests::tests::client::sync_detail_values(Default::default()).await
}

/// This test runs 3 mint transactions that get included in different blocks so that once we sync
/// we can check that each transaction gets marked as committed in the corresponding block.
#[tokio::test]
async fn multiple_transactions_can_be_committed_in_different_blocks_without_sync() {
    miden_client_integration_tests::tests::client::multiple_transactions_can_be_committed_in_different_blocks_without_sync(Default::default()).await
}

/// Test that checks multiple features:
/// - Consuming multiple notes in a single transaction.
/// - Consuming authenticated notes.
/// - Consuming unauthenticated notes.
#[tokio::test]
async fn consume_multiple_expected_notes() {
    miden_client_integration_tests::tests::client::consume_multiple_expected_notes(
        Default::default(),
    )
    .await
}

#[tokio::test]
async fn import_consumed_note_with_proof() {
    miden_client_integration_tests::tests::client::import_consumed_note_with_proof(
        Default::default(),
    )
    .await
}

#[tokio::test]
async fn import_consumed_note_with_id() {
    miden_client_integration_tests::tests::client::import_consumed_note_with_id(Default::default())
        .await
}

#[tokio::test]
async fn import_note_with_proof() {
    miden_client_integration_tests::tests::client::import_note_with_proof(Default::default()).await
}

#[tokio::test]
async fn discarded_transaction() {
    miden_client_integration_tests::tests::client::discarded_transaction(Default::default()).await
}

#[tokio::test]
async fn custom_transaction_prover() {
    miden_client_integration_tests::tests::client::custom_transaction_prover(Default::default())
        .await
}

#[tokio::test]
async fn locked_account() {
    miden_client_integration_tests::tests::client::locked_account(Default::default()).await
}

#[tokio::test]
async fn expired_transaction_fails() {
    miden_client_integration_tests::tests::client::expired_transaction_fails(Default::default())
        .await
}

/// Tests that RPC methods that are not directly related to the client logic
/// (like GetBlockByNumber) work correctly
#[tokio::test]
async fn unused_rpc_api() {
    miden_client_integration_tests::tests::client::unused_rpc_api(Default::default()).await
}

#[tokio::test]
async fn ignore_invalid_notes() {
    miden_client_integration_tests::tests::client::ignore_invalid_notes(Default::default()).await
}

#[tokio::test]
async fn output_only_note() {
    miden_client_integration_tests::tests::client::output_only_note(Default::default()).await
}
