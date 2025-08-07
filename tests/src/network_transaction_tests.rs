#[tokio::test]
async fn counter_contract_ntx() {
    miden_client_integration_tests::tests::network_transaction::counter_contract_ntx(
        Default::default(),
    )
    .await
}

#[tokio::test]
async fn recall_note_before_ntx_consumes_it() {
    miden_client_integration_tests::tests::network_transaction::recall_note_before_ntx_consumes_it(
        Default::default(),
    )
    .await
}
