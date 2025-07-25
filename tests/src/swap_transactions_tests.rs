#[tokio::test]
async fn swap_fully_onchain() {
    miden_client_integration_tests::tests::swap_transaction::swap_fully_onchain().await
}

#[tokio::test]
async fn swap_private() {
    miden_client_integration_tests::tests::swap_transaction::swap_private().await
}
