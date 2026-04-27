use miden_client::rpc::NodeRpcClient;
use miden_client::transaction::LocalTransactionProver;
use miden_testing::TxContextInput;

use crate::tests::create_test_client;

/// Exercises the mock `submit_proven_batch` path end-to-end: build a real
/// `ProvenBatch` from a proven transaction produced against a `MockChain`, submit it via
/// `MockRpcApi`, and verify the returned block number equals the chain tip. The mock
/// ignores `proposed_batch` and `transaction_inputs`, so we pass a cloned
/// `ProposedBatch` and an empty inputs vector — good enough to exercise the trait wiring.
#[tokio::test]
async fn submit_proven_batch_returns_chain_tip() {
    let (_client, rpc_api, _keystore) = Box::pin(create_test_client()).await;

    // Pick the first account recorded in the prebuilt mock chain.
    let account_id = rpc_api
        .mock_chain
        .read()
        .proven_blocks()
        .iter()
        .flat_map(|block| block.body().updated_accounts())
        .next()
        .unwrap()
        .account_id();

    // Execute and prove a trivial transaction against that account.
    let tx_context = rpc_api
        .mock_chain
        .read()
        .build_tx_context(TxContextInput::AccountId(account_id), &[], &[])
        .unwrap()
        .build()
        .unwrap();
    let executed_tx = Box::pin(tx_context.execute()).await.unwrap();

    let proven_tx = LocalTransactionProver::default().prove_dummy(executed_tx).unwrap();

    // Wrap the proven transaction into a ProvenBatch using MockChain helpers.
    // ProposedBatch is Clone, so we clone it before consuming the original to produce the
    // ProvenBatch.
    let (proven_batch, proposed_for_submit) = {
        let chain = rpc_api.mock_chain.read();
        let proposed_batch = chain.propose_transaction_batch(vec![proven_tx]).unwrap();
        let proposed_for_submit = proposed_batch.clone();
        let proven_batch = chain.prove_transaction_batch(proposed_batch).unwrap();
        (proven_batch, proposed_for_submit)
    };

    let expected_tip = rpc_api.get_chain_tip_block_num();
    let returned = Box::pin(rpc_api.submit_proven_batch(proven_batch, proposed_for_submit, vec![]))
        .await
        .unwrap();

    assert_eq!(returned, expected_tip);
}
