extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use miden_agglayer::claim_note::{Keccak256Output, ProofData, SmtNode};
use miden_agglayer::{EthAddressFormat, EthAmount, ExitRoot, GlobalIndex, LeafData, MetadataHash};
use miden_client::utils::hex_to_bytes;
use miden_protocol::account::AccountId;
use serde::Deserialize;

// SERDE HELPERS
// ================================================================================================

/// Deserializes a JSON value that may be either a number or a string into a `String`.
///
/// Foundry's `vm.serializeUint` outputs JSON numbers for uint256 values.
/// This deserializer accepts both `"100"` (string) and `100` (number) forms.
fn deserialize_uint_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        _ => Err(serde::de::Error::custom("expected a number or string for amount")),
    }
}

// TEST VECTOR TYPES
// ================================================================================================

/// Deserialized leaf value test vector from Solidity-generated JSON.
#[derive(Debug, Deserialize)]
pub struct LeafValueVector {
    pub origin_network: u32,
    pub origin_token_address: String,
    pub destination_network: u32,
    pub destination_address: String,
    #[serde(deserialize_with = "deserialize_uint_to_string")]
    pub amount: String,
    pub metadata_hash: String,
    #[allow(dead_code)]
    pub leaf_value: String,
}

impl LeafValueVector {
    /// Converts this test vector into a `LeafData` instance.
    pub fn to_leaf_data(&self) -> LeafData {
        LeafData {
            origin_network: self.origin_network,
            origin_token_address: EthAddressFormat::from_hex(&self.origin_token_address)
                .expect("valid origin token address hex"),
            destination_network: self.destination_network,
            destination_address: EthAddressFormat::from_hex(&self.destination_address)
                .expect("valid destination address hex"),
            amount: EthAmount::from_uint_str(&self.amount).expect("valid amount uint string"),
            metadata_hash: MetadataHash::new(
                hex_to_bytes(&self.metadata_hash).expect("valid metadata hash hex"),
            ),
        }
    }
}

/// Deserialized proof value test vector from Solidity-generated JSON.
/// Contains SMT proofs, exit roots, global index, and expected global exit root.
#[derive(Debug, Deserialize)]
pub struct ProofValueVector {
    pub smt_proof_local_exit_root: Vec<String>,
    pub smt_proof_rollup_exit_root: Vec<String>,
    pub global_index: String,
    pub mainnet_exit_root: String,
    pub rollup_exit_root: String,
    /// Expected global exit root: keccak256(mainnetExitRoot || rollupExitRoot)
    #[allow(dead_code)]
    pub global_exit_root: String,
}

impl ProofValueVector {
    /// Converts this test vector into a `ProofData` instance.
    pub fn to_proof_data(&self) -> ProofData {
        let smt_proof_local: [SmtNode; 32] = self
            .smt_proof_local_exit_root
            .iter()
            .map(|s| SmtNode::new(hex_to_bytes(s).expect("valid smt proof hex")))
            .collect::<Vec<_>>()
            .try_into()
            .expect("expected 32 SMT proof nodes for local exit root");

        let smt_proof_rollup: [SmtNode; 32] = self
            .smt_proof_rollup_exit_root
            .iter()
            .map(|s| SmtNode::new(hex_to_bytes(s).expect("valid smt proof hex")))
            .collect::<Vec<_>>()
            .try_into()
            .expect("expected 32 SMT proof nodes for rollup exit root");

        ProofData {
            smt_proof_local_exit_root: smt_proof_local,
            smt_proof_rollup_exit_root: smt_proof_rollup,
            global_index: GlobalIndex::from_hex(&self.global_index)
                .expect("valid global index hex"),
            mainnet_exit_root: Keccak256Output::new(
                hex_to_bytes(&self.mainnet_exit_root).expect("valid mainnet exit root hex"),
            ),
            rollup_exit_root: Keccak256Output::new(
                hex_to_bytes(&self.rollup_exit_root).expect("valid rollup exit root hex"),
            ),
        }
    }
}

/// Deserialized claim asset test vector from Solidity-generated JSON.
/// Contains both LeafData and ProofData from a real claimAsset transaction.
#[derive(Debug, Deserialize)]
pub struct ClaimAssetVector {
    #[serde(flatten)]
    pub proof: ProofValueVector,

    #[serde(flatten)]
    pub leaf: LeafValueVector,
}

// FOUNDRY TEST VECTOR GENERATION
// ================================================================================================

/// Path to the foundry project directory, relative to the crate's manifest directory.
const FOUNDRY_PROJECT_SUBDIR: &str = "foundry-vectors";

/// Path to the generated test vectors JSON file within the foundry project.
const FOUNDRY_OUTPUT_JSON: &str = "test-vectors/claim_asset_vectors_local_tx.json";

/// Runs the foundry test to generate claim asset test vectors for a given destination account.
///
/// This function:
/// 1. Converts the `AccountId` to an Ethereum address format (0x-prefixed hex)
/// 2. Invokes `forge test` with the `DESTINATION_ADDRESS` environment variable
/// 3. Reads and parses the generated JSON file
/// 4. Returns the `(ProofData, LeafData, ExitRoot)` tuple
///
/// # Panics
///
/// Panics if `forge` is not installed, the test fails, or the JSON output cannot be parsed.
pub fn generate_claim_data_for_account(account_id: AccountId) -> (ProofData, LeafData, ExitRoot) {
    let destination_address = EthAddressFormat::from_account_id(account_id);
    let destination_hex = destination_address.to_hex();
    println!(
        "[foundry] Generating claim data for account {:?} (eth address: {})",
        account_id, destination_hex
    );

    // Determine the foundry project directory using CARGO_MANIFEST_DIR.
    // This ensures the path is correct regardless of the test binary's working directory.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let foundry_dir = std::path::Path::new(manifest_dir).join(FOUNDRY_PROJECT_SUBDIR);
    assert!(
        foundry_dir.join("foundry.toml").exists(),
        "Foundry project not found at {}. Run `forge install` in that directory first.",
        foundry_dir.display()
    );

    // Ensure the test-vectors output directory exists (it is gitignored so may not
    // be present in a fresh checkout).
    let output_dir = foundry_dir.join("test-vectors");
    std::fs::create_dir_all(&output_dir).unwrap_or_else(|e| {
        panic!("failed to create test-vectors directory at {}: {}", output_dir.display(), e)
    });

    // Run forge test with the destination address as an environment variable
    let output = std::process::Command::new("forge")
        .arg("test")
        .arg("-vv")
        .arg("--match-contract")
        .arg("ClaimAssetTestVectorsLocalTx")
        .env("DESTINATION_ADDRESS", &destination_hex)
        .current_dir(&foundry_dir)
        .output()
        .expect("failed to execute `forge test` — is foundry installed?");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!("forge test failed!\nstdout:\n{}\nstderr:\n{}", stdout, stderr);
    }

    println!(
        "[foundry] forge test completed successfully:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );

    // Read and parse the generated JSON
    let json_path = foundry_dir.join(FOUNDRY_OUTPUT_JSON);
    let json_content = std::fs::read_to_string(&json_path).unwrap_or_else(|e| {
        panic!("failed to read generated test vectors from {}: {}", json_path.display(), e)
    });

    let vector: ClaimAssetVector = serde_json::from_str(&json_content)
        .expect("failed to parse foundry-generated claim asset vectors JSON");

    let ger = ExitRoot::new(
        hex_to_bytes(&vector.proof.global_exit_root).expect("valid global exit root hex"),
    );

    println!(
        "[foundry] Claim data generated successfully for destination: {}",
        destination_hex
    );

    (vector.proof.to_proof_data(), vector.leaf.to_leaf_data(), ger)
}
