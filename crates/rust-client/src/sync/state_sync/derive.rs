use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use miden_protocol::Word;
use miden_protocol::account::AccountId;
use miden_protocol::block::BlockNumber;
use miden_protocol::note::Nullifier;
use miden_protocol::transaction::InputNoteCommitment;

use crate::rpc::domain::transaction::{
    TransactionInclusion,
    TransactionRecord as RpcTransactionRecord,
};

// DERIVATION HELPERS
// ================================================================================================

/// Derives account commitment updates from transaction records.
///
/// For each unique account, takes the `final_state_commitment` from the transaction with the
/// highest `block_num`.
pub(crate) fn derive_account_commitment_updates(
    transaction_records: &[RpcTransactionRecord],
) -> Vec<(AccountId, Word)> {
    let mut latest_by_account: BTreeMap<AccountId, &RpcTransactionRecord> = BTreeMap::new();

    for record in transaction_records {
        let account_id = record.transaction_header.account_id();
        latest_by_account
            .entry(account_id)
            .and_modify(|existing| {
                if record.block_num > existing.block_num {
                    *existing = record;
                }
            })
            .or_insert(record);
    }

    latest_by_account
        .into_iter()
        .map(|(account_id, record)| {
            (account_id, record.transaction_header.final_state_commitment())
        })
        .collect()
}

/// Derives transaction inclusions and execution-ordered nullifiers from raw transaction records.
///
/// Returns the transaction inclusions (with nullifiers and output notes extracted from each
/// record's header) and nullifiers ordered by consuming transaction position per account.
pub(crate) fn derive_transaction_inclusions(
    transaction_records: Vec<RpcTransactionRecord>,
) -> (Vec<TransactionInclusion>, Vec<Nullifier>) {
    let ordered_nullifiers = compute_ordered_nullifiers(&transaction_records);

    let tx_inclusions = transaction_records
        .into_iter()
        .map(|r| {
            let nullifiers = r
                .transaction_header
                .input_notes()
                .iter()
                .map(InputNoteCommitment::nullifier)
                .collect();
            TransactionInclusion {
                transaction_id: r.transaction_header.id(),
                block_num: r.block_num,
                account_id: r.transaction_header.account_id(),
                initial_state_commitment: r.transaction_header.initial_state_commitment(),
                nullifiers,
                output_notes: r.output_notes,
            }
        })
        .collect();

    (tx_inclusions, ordered_nullifiers)
}

/// Returns nullifiers ordered by consuming transaction position, per account.
///
/// Groups RPC transaction records by (`account_id`, `block_num`), chains them using
/// `initial_state_commitment` / `final_state_commitment`, and collects each transaction's
/// input note nullifiers in execution order. Nullifiers from the same account are in execution
/// order; ordering across different accounts is arbitrary.
fn compute_ordered_nullifiers(transaction_records: &[RpcTransactionRecord]) -> Vec<Nullifier> {
    // Group transactions by (account_id, block_num).
    let mut groups: BTreeMap<(AccountId, BlockNumber), Vec<&RpcTransactionRecord>> =
        BTreeMap::new();

    for record in transaction_records {
        let account_id = record.transaction_header.account_id();
        groups.entry((account_id, record.block_num)).or_default().push(record);
    }

    let mut result = Vec::new();

    for txs in groups.values() {
        // Build a lookup from initial_state_commitment -> transaction record.
        let mut init_to_tx: BTreeMap<Word, &RpcTransactionRecord> = txs
            .iter()
            .map(|tx| (tx.transaction_header.initial_state_commitment(), *tx))
            .collect();

        // Build a set of all final states to find the chain start.
        let final_states: BTreeSet<Word> =
            txs.iter().map(|tx| tx.transaction_header.final_state_commitment()).collect();

        // Find the chain start: the tx whose initial_state_commitment is not any other tx's
        // final_state_commitment.
        let chain_start = txs
            .iter()
            .find(|tx| !final_states.contains(&tx.transaction_header.initial_state_commitment()));

        let Some(start_tx) = chain_start else {
            continue;
        };

        // Walk the chain from start, removing each step from the map.
        let mut current =
            init_to_tx.remove(&start_tx.transaction_header.initial_state_commitment());

        while let Some(tx) = current {
            for commitment in tx.transaction_header.input_notes().iter() {
                result.push(commitment.nullifier());
            }
            current = init_to_tx.remove(&tx.transaction_header.final_state_commitment());
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use miden_protocol::asset::FungibleAsset;
    use miden_protocol::block::BlockNumber;
    use miden_protocol::note::Nullifier;
    use miden_protocol::transaction::{InputNoteCommitment, InputNotes, TransactionHeader};
    use miden_protocol::{Felt, ZERO};

    use crate::rpc::domain::transaction::{
        ACCOUNT_ID_NATIVE_ASSET_FAUCET,
        TransactionRecord as RpcTransactionRecord,
    };

    fn word(n: u64) -> miden_protocol::Word {
        [Felt::new(n), ZERO, ZERO, ZERO].into()
    }

    fn make_rpc_tx(
        init_state: u64,
        final_state: u64,
        nullifier_vals: &[u64],
        block_number: u32,
    ) -> RpcTransactionRecord {
        let account_id = miden_protocol::account::AccountId::try_from(
            miden_protocol::testing::account_id::ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE,
        )
        .unwrap();

        let input_notes = InputNotes::new_unchecked(
            nullifier_vals
                .iter()
                .map(|v| InputNoteCommitment::from(Nullifier::from_raw(word(*v))))
                .collect(),
        );

        let fee =
            FungibleAsset::new(ACCOUNT_ID_NATIVE_ASSET_FAUCET.try_into().expect("valid"), 0u64)
                .unwrap();

        RpcTransactionRecord {
            block_num: BlockNumber::from(block_number),
            transaction_header: TransactionHeader::new(
                account_id,
                word(init_state),
                word(final_state),
                input_notes,
                vec![],
                fee,
            ),
            output_notes: vec![],
        }
    }

    #[test]
    fn chains_rpc_transactions_by_state_commitment() {
        let tx_a = make_rpc_tx(1, 2, &[10], 5);
        let tx_b = make_rpc_tx(2, 3, &[20], 5);
        let tx_c = make_rpc_tx(3, 4, &[30], 5);

        let result = super::compute_ordered_nullifiers(&[tx_c, tx_a, tx_b]);

        assert_eq!(result[0], Nullifier::from_raw(word(10)));
        assert_eq!(result[1], Nullifier::from_raw(word(20)));
        assert_eq!(result[2], Nullifier::from_raw(word(30)));
    }

    #[test]
    fn groups_independently_by_account_and_block() {
        let tx_a1 = make_rpc_tx(1, 2, &[10], 5);
        let tx_a2 = make_rpc_tx(2, 3, &[20], 5);
        let tx_a3 = make_rpc_tx(3, 4, &[30], 6);

        let account_b = miden_protocol::account::AccountId::try_from(
            miden_protocol::testing::account_id::ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET,
        )
        .unwrap();

        let fee =
            FungibleAsset::new(ACCOUNT_ID_NATIVE_ASSET_FAUCET.try_into().expect("valid"), 0u64)
                .unwrap();

        let tx_b1 = RpcTransactionRecord {
            block_num: BlockNumber::from(5u32),
            transaction_header: TransactionHeader::new(
                account_b,
                word(100),
                word(200),
                InputNotes::new_unchecked(vec![InputNoteCommitment::from(Nullifier::from_raw(
                    word(40),
                ))]),
                vec![],
                fee,
            ),
            output_notes: vec![],
        };

        let result = super::compute_ordered_nullifiers(&[tx_a2, tx_b1, tx_a3, tx_a1]);

        let pos = |val: u64| -> usize {
            result.iter().position(|n| *n == Nullifier::from_raw(word(val))).unwrap()
        };

        assert!(pos(10) < pos(20));
        assert!(result.contains(&Nullifier::from_raw(word(30))));
        assert!(result.contains(&Nullifier::from_raw(word(40))));
    }

    #[test]
    fn multiple_nullifiers_per_transaction_are_consecutive() {
        let tx = make_rpc_tx(1, 2, &[10, 20, 30], 5);

        let result = super::compute_ordered_nullifiers(&[tx]);

        assert_eq!(result.len(), 3);
        assert!(result.contains(&Nullifier::from_raw(word(10))));
        assert!(result.contains(&Nullifier::from_raw(word(20))));
        assert!(result.contains(&Nullifier::from_raw(word(30))));
    }

    #[test]
    fn empty_input_returns_empty_vec() {
        let result = super::compute_ordered_nullifiers(&[]);
        assert!(result.is_empty());
    }
}
