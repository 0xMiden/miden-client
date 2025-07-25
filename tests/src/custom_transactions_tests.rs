// CUSTOM TRANSACTION REQUEST
// ================================================================================================
//
// The following functions are for testing custom transaction code. What the test does is:
//
// - Create a custom tx that mints a custom note which checks that the note args are as expected
//   (ie, a word of 8 felts that represent [9, 12, 18, 3, 3, 18, 12, 9])
//      - The args will be provided via the advice map
//
// - Create another transaction that consumes this note with custom code. This custom code only
//   asserts that the {asserted_value} parameter is 0. To test this we first execute with an
//   incorrect value passed in, and after that we try again with the correct value.
//
// Because it's currently not possible to create/consume notes without assets, the P2ID code
// is used as the base for the note code.

#[tokio::test]
async fn transaction_request() {
    miden_client_integration_tests::tests::custom_transaction::transaction_request().await
}

#[tokio::test]
async fn merkle_store() {
    miden_client_integration_tests::tests::custom_transaction::merkle_store().await
}

#[tokio::test]
async fn onchain_notes_sync_with_tag() {
    miden_client_integration_tests::tests::custom_transaction::onchain_notes_sync_with_tag().await
}
