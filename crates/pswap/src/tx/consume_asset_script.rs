use alloc::vec::Vec;

use miden_core::crypto::hash::Rpo256;
use miden_crypto::utils::Deserializable;
use miden_mast_package::Package;
use miden_protocol::asset::Asset;
use miden_protocol::transaction::TransactionScript;
use miden_protocol::utils::sync::LazyLock;
use miden_protocol::{Felt, Word};

// TX SCRIPT
// ================================================================================================

const CONSUME_ASSET_SCRIPT_BYTES: &[u8] =
    include_bytes!("../../../../contracts/consume-asset/consume_asset.masp");

/// Load the compiled consume-asset-script program once.
static CONSUME_ASSET_SCRIPT_PROGRAM: LazyLock<TransactionScript> = LazyLock::new(|| {
    let package = Package::read_from_bytes(CONSUME_ASSET_SCRIPT_BYTES)
        .expect("Failed to deserialize consume-asset-script package");
    let program = package.unwrap_program();
    TransactionScript::from_parts(program.mast_forest().clone(), program.entrypoint())
});

// CONSUME ASSET SCRIPT
// ================================================================================================

/// Output of [`ConsumeAssetScript::prepare`]: everything needed to wire
/// the consume-asset-script into a transaction context.
pub struct ConsumeAssetData {
    /// The commitment word passed as the tx-script argument.
    pub commitment_arg: Word,
    /// `(key, value)` pair to feed into `extend_advice_map`.
    pub advice_map_entry: (Word, Vec<Felt>),
}

/// SDK helper for the **consume-asset-script**: consumes spread assets directly
/// into the executing account's vault.
///
/// # Usage
///
/// ```ignore
/// use pswap::ConsumeAssetScript;
///
/// // 1. Get the TransactionScript
/// let tx_script = ConsumeAssetScript::tx_script();
///
/// // 2. Prepare advice data
/// let data = ConsumeAssetScript::prepare(&[asset1, asset2]);
///
/// // 3. Wire into the transaction context
/// let tx_context = mock_chain
///     .build_tx_context(sender_id, &input_note_ids, &[])?
///     .tx_script(tx_script)
///     .tx_script_args(data.commitment_arg)
///     .extend_advice_map([data.advice_map_entry])
///     .build()?;
/// ```
pub struct ConsumeAssetScript;

impl ConsumeAssetScript {
    /// Returns the compiled `TransactionScript` for the consume-asset-script.
    pub fn tx_script() -> TransactionScript {
        CONSUME_ASSET_SCRIPT_PROGRAM.clone()
    }

    /// Prepares the commitment and advice map for consuming spread assets
    /// directly into the executing account's vault.
    ///
    /// # Arguments
    ///
    /// * `assets` – slice of assets to consume as spread.
    ///
    /// # Returns
    ///
    /// An [`ConsumeAssetData`] containing the commitment arg and advice map entry.
    pub fn prepare(assets: &[Asset]) -> ConsumeAssetData {
        assert!(!assets.is_empty(), "must provide at least one asset");

        let mut advice_felts: Vec<Felt> = Vec::with_capacity(assets.len() * 4);

        for asset in assets {
            let asset_word = Word::from(*asset);
            advice_felts.extend(asset_word);
        }

        // Compute RPO commitment over all advice felts
        let commitment_key: Word = Rpo256::hash_elements(&advice_felts);
        let mut commitment_arg = commitment_key;
        commitment_arg.reverse();

        ConsumeAssetData {
            commitment_arg,
            advice_map_entry: (commitment_key, advice_felts),
        }
    }
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_script_loads() {
        let _script = ConsumeAssetScript::tx_script();
    }
}
