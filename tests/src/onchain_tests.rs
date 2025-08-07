#[tokio::test]
async fn onchain_notes_flow() {
    miden_client_integration_tests::tests::onchain::onchain_notes_flow(Default::default()).await
}

#[tokio::test]
async fn onchain_accounts() {
    miden_client_integration_tests::tests::onchain::onchain_accounts(Default::default()).await
}

#[tokio::test]
async fn import_account_by_id() {
    miden_client_integration_tests::tests::onchain::import_account_by_id(Default::default()).await
}

#[tokio::test]
async fn incorrect_genesis() {
    miden_client_integration_tests::tests::onchain::incorrect_genesis(Default::default()).await
}
