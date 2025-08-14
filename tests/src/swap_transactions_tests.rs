#[tokio::test]
async fn swap_fully_onchain() {
    miden_client_integration_tests::tests::swap_transaction::swap_fully_onchain(Default::default())
        .await
}

#[tokio::test]
async fn swap_private() {
    miden_client_integration_tests::tests::swap_transaction::swap_private(Default::default()).await
}
