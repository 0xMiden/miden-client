mod integration_tests {
    use miden_client::testing::config::ClientConfig;
    use miden_client_integration_tests::tests::{
        client,
        custom_transaction,
        fpi,
        network_transaction,
        onchain,
        swap_transaction,
    };

    // Test wrappers moved from root tests directory
    // These provide the #[tokio::test] interface for running individual tests
    #[tokio::test]
    async fn client_builder_initializes_client_with_endpoint() {
        client::client_builder_initializes_client_with_endpoint(Default::default()).await;
    }

    #[tokio::test]
    async fn multiple_tx_on_same_block() {
        client::multiple_tx_on_same_block(Default::default()).await;
    }

    #[tokio::test]
    async fn import_expected_notes() {
        client::import_expected_notes(Default::default()).await;
    }

    #[tokio::test]
    async fn import_expected_note_uncommitted() {
        client::import_expected_note_uncommitted(Default::default()).await;
    }

    #[tokio::test]
    async fn import_expected_notes_from_the_past_as_committed() {
        client::import_expected_notes_from_the_past_as_committed(Default::default()).await;
    }

    #[tokio::test]
    async fn get_account_update() {
        client::get_account_update(Default::default()).await;
    }

    #[tokio::test]
    async fn sync_detail_values() {
        client::sync_detail_values(Default::default()).await;
    }

    #[tokio::test]
    async fn multiple_transactions_can_be_committed_in_different_blocks_without_sync() {
        client::multiple_transactions_can_be_committed_in_different_blocks_without_sync(
            Default::default(),
        )
        .await;
    }

    #[tokio::test]
    async fn consume_multiple_expected_notes() {
        client::consume_multiple_expected_notes(Default::default()).await;
    }

    #[tokio::test]
    async fn import_consumed_note_with_proof() {
        client::import_consumed_note_with_proof(Default::default()).await;
    }

    #[tokio::test]
    async fn import_consumed_note_with_id() {
        client::import_consumed_note_with_id(Default::default()).await;
    }

    #[tokio::test]
    async fn import_note_with_proof() {
        client::import_note_with_proof(Default::default()).await;
    }

    #[tokio::test]
    async fn discarded_transaction() {
        client::discarded_transaction(Default::default()).await;
    }

    #[tokio::test]
    async fn custom_transaction_prover() {
        client::custom_transaction_prover(Default::default()).await;
    }

    #[tokio::test]
    async fn locked_account() {
        client::locked_account(Default::default()).await;
    }

    #[tokio::test]
    async fn expired_transaction_fails() {
        client::expired_transaction_fails(Default::default()).await;
    }

    #[tokio::test]
    async fn unused_rpc_api() {
        client::unused_rpc_api(ClientConfig::default()).await;
    }

    #[tokio::test]
    async fn ignore_invalid_notes() {
        client::ignore_invalid_notes(ClientConfig::default()).await;
    }

    #[tokio::test]
    async fn output_only_note() {
        client::output_only_note(ClientConfig::default()).await;
    }

    #[tokio::test]
    async fn transaction_request() {
        custom_transaction::transaction_request(ClientConfig::default()).await;
    }

    #[tokio::test]
    async fn merkle_store() {
        custom_transaction::merkle_store(ClientConfig::default()).await;
    }

    #[tokio::test]
    async fn onchain_notes_sync_with_tag() {
        custom_transaction::onchain_notes_sync_with_tag(ClientConfig::default()).await;
    }

    #[tokio::test]
    async fn standard_fpi_public() {
        fpi::standard_fpi_public(ClientConfig::default()).await;
    }

    #[tokio::test]
    async fn standard_fpi_private() {
        fpi::standard_fpi_private(ClientConfig::default()).await;
    }

    #[tokio::test]
    async fn fpi_execute_program() {
        fpi::fpi_execute_program(ClientConfig::default()).await;
    }

    #[tokio::test]
    async fn nested_fpi_calls() {
        fpi::nested_fpi_calls(ClientConfig::default()).await;
    }

    #[tokio::test]
    async fn counter_contract_ntx() {
        network_transaction::counter_contract_ntx(ClientConfig::default()).await;
    }

    #[tokio::test]
    async fn recall_note_before_ntx_consumes_it() {
        network_transaction::recall_note_before_ntx_consumes_it(ClientConfig::default()).await;
    }

    #[tokio::test]
    async fn import_account_by_id() {
        onchain::import_account_by_id(ClientConfig::default()).await;
    }

    #[tokio::test]
    async fn onchain_accounts() {
        onchain::onchain_accounts(ClientConfig::default()).await;
    }

    #[tokio::test]
    async fn onchain_notes_flow() {
        onchain::onchain_notes_flow(ClientConfig::default()).await;
    }

    #[tokio::test]
    async fn incorrect_genesis() {
        onchain::incorrect_genesis(ClientConfig::default()).await;
    }

    #[tokio::test]
    async fn swap_fully_onchain() {
        swap_transaction::swap_fully_onchain(ClientConfig::default()).await;
    }

    #[tokio::test]
    async fn swap_private() {
        swap_transaction::swap_private(ClientConfig::default()).await;
    }
}
