mod integration_tests {
    use miden_client_integration_tests::tests::config::ClientConfig;
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
    async fn client_builder_initializes_client_with_endpoint() -> anyhow::Result<()> {
        client::client_builder_initializes_client_with_endpoint(Default::default()).await
    }

    #[tokio::test]
    async fn multiple_tx_on_same_block() -> anyhow::Result<()> {
        client::multiple_tx_on_same_block(Default::default()).await
    }

    #[tokio::test]
    async fn import_expected_notes() -> anyhow::Result<()> {
        client::import_expected_notes(Default::default()).await
    }

    #[tokio::test]
    async fn import_expected_note_uncommitted() -> anyhow::Result<()> {
        client::import_expected_note_uncommitted(Default::default()).await
    }

    #[tokio::test]
    async fn import_expected_notes_from_the_past_as_committed() -> anyhow::Result<()> {
        client::import_expected_notes_from_the_past_as_committed(Default::default()).await
    }

    #[tokio::test]
    async fn get_account_update() -> anyhow::Result<()> {
        client::get_account_update(Default::default()).await
    }

    #[tokio::test]
    async fn sync_detail_values() -> anyhow::Result<()> {
        client::sync_detail_values(Default::default()).await
    }

    #[tokio::test]
    async fn multiple_transactions_can_be_committed_in_different_blocks_without_sync()
    -> anyhow::Result<()> {
        client::multiple_transactions_can_be_committed_in_different_blocks_without_sync(
            Default::default(),
        )
        .await
    }

    #[tokio::test]
    async fn consume_multiple_expected_notes() -> anyhow::Result<()> {
        client::consume_multiple_expected_notes(Default::default()).await
    }

    #[tokio::test]
    async fn import_consumed_note_with_proof() -> anyhow::Result<()> {
        client::import_consumed_note_with_proof(Default::default()).await
    }

    #[tokio::test]
    async fn import_consumed_note_with_id() -> anyhow::Result<()> {
        client::import_consumed_note_with_id(Default::default()).await
    }

    #[tokio::test]
    async fn import_note_with_proof() -> anyhow::Result<()> {
        client::import_note_with_proof(Default::default()).await
    }

    #[tokio::test]
    async fn discarded_transaction() -> anyhow::Result<()> {
        client::discarded_transaction(Default::default()).await
    }

    #[tokio::test]
    async fn custom_transaction_prover() -> anyhow::Result<()> {
        client::custom_transaction_prover(Default::default()).await
    }

    #[tokio::test]
    async fn locked_account() -> anyhow::Result<()> {
        client::locked_account(Default::default()).await
    }

    #[tokio::test]
    async fn expired_transaction_fails() -> anyhow::Result<()> {
        client::expired_transaction_fails(Default::default()).await
    }

    #[tokio::test]
    async fn unused_rpc_api() -> anyhow::Result<()> {
        client::unused_rpc_api(ClientConfig::default()).await
    }

    #[tokio::test]
    async fn ignore_invalid_notes() -> anyhow::Result<()> {
        client::ignore_invalid_notes(ClientConfig::default()).await
    }

    #[tokio::test]
    async fn output_only_note() -> anyhow::Result<()> {
        client::output_only_note(ClientConfig::default()).await
    }

    #[tokio::test]
    async fn transaction_request() -> anyhow::Result<()> {
        custom_transaction::transaction_request(ClientConfig::default()).await
    }

    #[tokio::test]
    async fn merkle_store() -> anyhow::Result<()> {
        custom_transaction::merkle_store(ClientConfig::default()).await
    }

    #[tokio::test]
    async fn onchain_notes_sync_with_tag() -> anyhow::Result<()> {
        custom_transaction::onchain_notes_sync_with_tag(ClientConfig::default()).await
    }

    #[tokio::test]
    async fn standard_fpi_public() -> anyhow::Result<()> {
        fpi::standard_fpi_public(ClientConfig::default()).await
    }

    #[tokio::test]
    async fn standard_fpi_private() -> anyhow::Result<()> {
        fpi::standard_fpi_private(ClientConfig::default()).await
    }

    #[tokio::test]
    async fn fpi_execute_program() -> anyhow::Result<()> {
        fpi::fpi_execute_program(ClientConfig::default()).await
    }

    #[tokio::test]
    async fn nested_fpi_calls() -> anyhow::Result<()> {
        fpi::nested_fpi_calls(ClientConfig::default()).await
    }

    #[tokio::test]
    async fn counter_contract_ntx() -> anyhow::Result<()> {
        network_transaction::counter_contract_ntx(ClientConfig::default()).await
    }

    #[tokio::test]
    async fn recall_note_before_ntx_consumes_it() -> anyhow::Result<()> {
        network_transaction::recall_note_before_ntx_consumes_it(ClientConfig::default()).await
    }

    #[tokio::test]
    async fn import_account_by_id() -> anyhow::Result<()> {
        onchain::import_account_by_id(ClientConfig::default()).await
    }

    #[tokio::test]
    async fn onchain_accounts() -> anyhow::Result<()> {
        onchain::onchain_accounts(ClientConfig::default()).await
    }

    #[tokio::test]
    async fn onchain_notes_flow() -> anyhow::Result<()> {
        onchain::onchain_notes_flow(ClientConfig::default()).await
    }

    #[tokio::test]
    async fn incorrect_genesis() -> anyhow::Result<()> {
        onchain::incorrect_genesis(ClientConfig::default()).await
    }

    #[tokio::test]
    async fn swap_fully_onchain() -> anyhow::Result<()> {
        swap_transaction::swap_fully_onchain(ClientConfig::default()).await
    }

    #[tokio::test]
    async fn swap_private() -> anyhow::Result<()> {
        swap_transaction::swap_private(ClientConfig::default()).await
    }
}
