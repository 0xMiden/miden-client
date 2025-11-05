use miden_client::account::AccountType as NativeAccountType;
use wasm_bindgen::prelude::*;

/// Enumerates the account types supported by the Miden network.
#[derive(Clone)]
#[wasm_bindgen]
pub enum AccountType {
    /// Faucet that mints fungible assets.
    FungibleFaucet,
    /// Faucet that mints non-fungible assets.
    NonFungibleFaucet,
    /// Regular account with code that cannot be updated.
    RegularAccountImmutableCode,
    /// Regular account whose code can be updated.
    RegularAccountUpdatableCode,
}

// CONVERSIONS
// ================================================================================================

impl From<AccountType> for NativeAccountType {
    fn from(value: AccountType) -> Self {
        match value {
            AccountType::FungibleFaucet => NativeAccountType::FungibleFaucet,
            AccountType::NonFungibleFaucet => NativeAccountType::NonFungibleFaucet,
            AccountType::RegularAccountImmutableCode => {
                NativeAccountType::RegularAccountImmutableCode
            },
            AccountType::RegularAccountUpdatableCode => {
                NativeAccountType::RegularAccountUpdatableCode
            },
        }
    }
}

impl From<&AccountType> for NativeAccountType {
    fn from(value: &AccountType) -> Self {
        match value {
            AccountType::FungibleFaucet => NativeAccountType::FungibleFaucet,
            AccountType::NonFungibleFaucet => NativeAccountType::NonFungibleFaucet,
            AccountType::RegularAccountImmutableCode => {
                NativeAccountType::RegularAccountImmutableCode
            },
            AccountType::RegularAccountUpdatableCode => {
                NativeAccountType::RegularAccountUpdatableCode
            },
        }
    }
}
