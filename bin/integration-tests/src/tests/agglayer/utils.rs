use anyhow::Result;
use miden_agglayer::{AggLayerBridge, ExitRoot};
use miden_client::account::Account;
use miden_client::crypto::Rpo256;
use miden_client::{Felt, ONE, Word, ZERO};

const REGISTERED_GER_MAP_VALUE: Word = Word::new([ONE, ZERO, ZERO, ZERO]);

/// Returns a boolean indicating whether the provided GER is present in storage of the provided
/// bridge account.
pub fn is_ger_registered(ger: ExitRoot, bridge_account: Account) -> Result<bool> {
    // Compute the expected GER hash: rpo256::merge(GER_UPPER, GER_LOWER)
    let mut ger_lower: [Felt; 4] = ger.to_elements()[0..4].try_into().unwrap();
    let mut ger_upper: [Felt; 4] = ger.to_elements()[4..8].try_into().unwrap();
    // Elements are reversed: rpo256::merge treats stack as if loaded BE from memory
    // The following will produce matching hashes:
    // Rust
    // Hasher::merge(&[a, b, c, d], &[e, f, g, h])
    // MASM
    // rpo256::merge(h, g, f, e, d, c, b, a)
    ger_lower.reverse();
    ger_upper.reverse();
    let ger_hash = Rpo256::merge(&[ger_upper.into(), ger_lower.into()]);

    // Get the value stored by the GER hash. If this GER was registered, the value would be equal to
    // [1, 0, 0, 0]
    let stored_value = bridge_account
        .storage()
        .get_map_item(AggLayerBridge::ger_map_slot_name(), ger_hash)?;

    if stored_value == REGISTERED_GER_MAP_VALUE {
        Ok(true)
    } else {
        Ok(false)
    }
}
