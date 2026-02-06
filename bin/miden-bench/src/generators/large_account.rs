#![allow(clippy::cast_possible_truncation, clippy::cast_lossless)]

use std::fmt::Write;

use miden_client::assembly::CodeBuilder;
use miden_protocol::Felt;
use miden_protocol::account::auth::AuthSecretKey;
use miden_protocol::account::{
    Account,
    AccountBuilder,
    AccountComponent,
    AccountStorageMode,
    AccountType,
    StorageMap,
    StorageSlot,
    StorageSlotName,
};
use miden_standards::account::auth::AuthFalcon512Rpo;
use miden_standards::account::components::basic_wallet_library;
use rand::Rng;
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;

/// Configuration for generating large accounts
#[derive(Clone, Debug)]
pub struct LargeAccountConfig {
    /// Number of entries per storage map
    pub num_storage_map_entries: usize,
    /// Number of map-type storage slots
    pub num_map_slots: usize,
    /// Seed for deterministic generation
    pub seed: [u8; 32],
}

impl LargeAccountConfig {
    /// Creates a new configuration with the specified number of maps and entries per map
    pub fn new(maps: usize, entries_per_map: usize) -> Self {
        let mut rng = rand::rng();
        let mut seed = [0u8; 32];
        rng.fill(&mut seed);

        Self {
            num_storage_map_entries: entries_per_map,
            num_map_slots: maps,
            seed,
        }
    }

    /// Creates a new configuration with a specific seed (for deterministic generation)
    #[allow(dead_code)]
    pub fn with_seed(maps: usize, entries_per_map: usize, seed: [u8; 32]) -> Self {
        Self {
            num_storage_map_entries: entries_per_map,
            num_map_slots: maps,
            seed,
        }
    }

    /// Returns the total number of storage entries
    pub fn total_entries(&self) -> usize {
        self.num_map_slots * self.num_storage_map_entries
    }
}

/// Generates a deterministic seed from an index
#[allow(dead_code)]
pub fn generate_deterministic_seed(index: u32) -> [u8; 32] {
    let mut seed = [0u8; 32];
    let bytes = index.to_le_bytes();
    seed[0..4].copy_from_slice(&bytes);
    seed[4..8].copy_from_slice(&bytes);
    seed[8..12].copy_from_slice(&bytes);
    seed[12..16].copy_from_slice(&bytes);
    seed
}

/// Generates MASM code for a storage reader component with `get_map_item_slot_N` procedures.
///
/// Each procedure reads a map item from the corresponding storage slot. These procedures
/// must be called from within account context (via `call` from a transaction script), because
/// the kernel's `get_map_item` handler verifies the caller is an account procedure.
pub fn generate_reader_component_code(num_slots: usize) -> String {
    let mut code = String::new();

    for i in 0..num_slots {
        let slot_name = format!("miden::bench::map_slot_{i}");
        write!(
            code,
            r#"const MAP_SLOT_{i} = word("{slot_name}")

# Reads an item from storage slot {i}.
# Stack input: [KEY]
# Stack output: [VALUE]
pub proc get_map_item_slot_{i}
    push.MAP_SLOT_{i}[0..2]
    exec.::miden::protocol::active_account::get_map_item
end

"#
        )
        .expect("writing to String should not fail");
    }

    code
}

/// Creates a large account with the specified configuration
pub fn create_large_account(
    config: &LargeAccountConfig,
) -> anyhow::Result<(Account, AuthSecretKey)> {
    let sk = AuthSecretKey::new_falcon512_rpo_with_rng(&mut ChaCha20Rng::from_seed(config.seed));

    // Create storage map slots
    let mut storage_slots = Vec::new();
    for i in 0..config.num_map_slots {
        let slot_name = format!("miden::bench::map_slot_{i}");
        storage_slots.push(create_large_storage_slot(
            slot_name.as_str(),
            config.num_storage_map_entries,
            i as u32,
        ));
    }

    // Reader component: owns storage slots and provides get_map_item_slot_N procedures
    let reader_code = generate_reader_component_code(config.num_map_slots);
    let reader_component_code = CodeBuilder::default()
        .compile_component_code("miden::bench::storage_reader", &reader_code)
        .map_err(|e| anyhow::anyhow!("Failed to compile reader component: {e}"))?;
    let reader_component = AccountComponent::new(reader_component_code, storage_slots)
        .map_err(|e| anyhow::anyhow!("Failed to create reader component: {e}"))?
        .with_supports_all_types();

    // Wallet component: provides standard wallet operations (no storage slots)
    let wallet_component = AccountComponent::new(basic_wallet_library(), vec![])
        .expect("basic wallet component should satisfy account component requirements")
        .with_supports_all_types();

    let account = AccountBuilder::new(config.seed)
        .with_auth_component(AuthFalcon512Rpo::new(sk.public_key().to_commitment()))
        .account_type(AccountType::RegularAccountUpdatableCode)
        .with_component(wallet_component)
        .with_component(reader_component)
        .storage_mode(AccountStorageMode::Public)
        .build()?;

    Ok((account, sk))
}

/// Generates a random non-zero `[Felt; 4]` value suitable for storage map entries.
///
/// Values must be non-zero because the SMT treats zero values as deletions.
/// The probability of generating an all-zero word is astronomically small (~2^-256),
/// but we guard against it for correctness.
pub fn random_word(rng: &mut impl Rng) -> [Felt; 4] {
    loop {
        let word: [Felt; 4] = std::array::from_fn(|_| Felt::new(rng.random::<u64>() >> 1));
        if word.iter().any(|f| f.as_int() != 0) {
            return word;
        }
    }
}

/// Creates an RNG seeded from a slot index, for deterministic random value generation.
pub fn slot_rng(seed: u32) -> ChaCha20Rng {
    let mut rng_seed = [0u8; 32];
    rng_seed[0..4].copy_from_slice(&seed.to_le_bytes());
    ChaCha20Rng::from_seed(rng_seed)
}

/// Creates a storage slot with many map entries
pub fn create_large_storage_slot(name: &str, num_entries: usize, seed: u32) -> StorageSlot {
    let mut rng = slot_rng(seed);

    let map_entries: Vec<_> = (0..num_entries as u32)
        .map(|i| {
            let key_val = seed.wrapping_mul(1000).wrapping_add(i);
            let key = [Felt::new(key_val as u64); 4];
            let value = random_word(&mut rng);
            (key.into(), value.into())
        })
        .collect();

    StorageSlot::with_map(
        StorageSlotName::new(name).expect("slot name should be valid"),
        StorageMap::with_entries(map_entries).expect("map entries should be valid"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_small_account() {
        let config = LargeAccountConfig::with_seed(1, 10, [0x01; 32]);
        let result = create_large_account(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_large_storage_slot() {
        let slot = create_large_storage_slot("test::slot", 100, 0);
        assert!(matches!(slot.slot_type(), miden_protocol::account::StorageSlotType::Map));
    }
}
