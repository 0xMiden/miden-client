#[tokio::test]
async fn onchain_notes_flow() {
    miden_client_integration_tests::tests::onchain::onchain_notes_flow().await
}

#[tokio::test]
async fn onchain_accounts() {
    miden_client_integration_tests::tests::onchain::onchain_accounts().await
}

#[tokio::test]
async fn incorrect_genesis() {
    miden_client_integration_tests::tests::onchain::incorrect_genesis().await
}
