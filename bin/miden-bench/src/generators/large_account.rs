#![allow(clippy::cast_possible_truncation, clippy::cast_lossless)]

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
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;

use crate::AccountSize;

/// Configuration for generating large accounts
#[derive(Clone, Debug)]
pub struct LargeAccountConfig {
    /// Number of value-type storage slots
    pub num_storage_slots: usize,
    /// Number of entries per storage map
    pub num_storage_map_entries: usize,
    /// Number of map-type storage slots
    pub num_map_slots: usize,
    /// Seed for deterministic generation
    pub seed: [u8; 32],
}

impl LargeAccountConfig {
    /// Small configuration for baseline measurements (10 entries)
    pub const SMALL: Self = Self {
        num_storage_slots: 2,
        num_storage_map_entries: 10,
        num_map_slots: 1,
        seed: [0x01; 32],
    };

    /// Medium configuration for typical usage (100 entries)
    pub const MEDIUM: Self = Self {
        num_storage_slots: 5,
        num_storage_map_entries: 100,
        num_map_slots: 1,
        seed: [0x02; 32],
    };

    /// Large configuration for stress testing (1000 entries)
    pub const LARGE: Self = Self {
        num_storage_slots: 10,
        num_storage_map_entries: 1000,
        num_map_slots: 1,
        seed: [0x03; 32],
    };

    /// Very large configuration for extreme stress testing (50000 entries)
    pub const VERY_LARGE: Self = Self {
        num_storage_slots: 20,
        num_storage_map_entries: 50000,
        num_map_slots: 1,
        seed: [0x04; 32],
    };

    /// Creates a configuration from an `AccountSize`
    pub fn from_size(size: AccountSize) -> Self {
        match size {
            AccountSize::Small => Self::SMALL,
            AccountSize::Medium => Self::MEDIUM,
            AccountSize::Large => Self::LARGE,
            AccountSize::VeryLarge => Self::VERY_LARGE,
        }
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

/// Creates a large account with the specified configuration
pub fn create_large_account(
    config: &LargeAccountConfig,
) -> anyhow::Result<(Account, AuthSecretKey)> {
    let sk = AuthSecretKey::new_falcon512_rpo_with_rng(&mut ChaCha20Rng::from_seed(config.seed));

    // Create storage slots
    let mut storage_slots = Vec::new();

    // Add value slots
    for i in 0..config.num_storage_slots {
        let slot_name = format!("miden::bench::value_slot_{i}");
        let value = [Felt::new(i as u64); 4];
        storage_slots.push(StorageSlot::with_value(
            StorageSlotName::new(slot_name.as_str()).expect("slot name should be valid"),
            value.into(),
        ));
    }

    // Add map slots
    for i in 0..config.num_map_slots {
        let slot_name = format!("miden::bench::map_slot_{i}");
        storage_slots.push(create_large_storage_slot(
            slot_name.as_str(),
            config.num_storage_map_entries,
            i as u32,
        ));
    }

    let acc_component = AccountComponent::new(basic_wallet_library(), storage_slots)
        .expect("basic wallet component should satisfy account component requirements")
        .with_supports_all_types();

    let account = AccountBuilder::new(config.seed)
        .with_auth_component(AuthFalcon512Rpo::new(sk.public_key().to_commitment()))
        .account_type(AccountType::RegularAccountUpdatableCode)
        .with_component(acc_component)
        .storage_mode(AccountStorageMode::Public)
        .build()?;

    Ok((account, sk))
}

/// Creates a storage slot with many map entries
pub fn create_large_storage_slot(name: &str, num_entries: usize, seed: u32) -> StorageSlot {
    let map_entries = (0..num_entries as u32).map(|i| {
        let key_val = seed.wrapping_mul(1000).wrapping_add(i);
        let key = [Felt::new(key_val as u64); 4];
        let value = [Felt::new(i as u64); 4];
        (key.into(), value.into())
    });

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
        let config = LargeAccountConfig::SMALL;
        let result = create_large_account(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_large_storage_slot() {
        let slot = create_large_storage_slot("test::slot", 100, 0);
        assert!(matches!(slot.slot_type(), miden_protocol::account::StorageSlotType::Map));
    }
}
