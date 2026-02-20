use miden_client::account::AccountType as NativeAccountType;

use crate::prelude::*;

// The enum definition differs between wasm (repr(u8)) and napi (string_enum),
// so we use separate cfg blocks.

#[cfg(feature = "wasm")]
mod def {
    use super::*;

    #[derive(Clone)]
    #[bindings]
    pub enum AccountType {
        FungibleFaucet,
        NonFungibleFaucet,
        RegularAccountImmutableCode,
        RegularAccountUpdatableCode,
    }
}

#[cfg(feature = "napi")]
mod def {
    use napi_derive::napi;

    #[bindings(string_enum)]
    pub enum AccountType {
        FungibleFaucet,
        NonFungibleFaucet,
        RegularAccountImmutableCode,
        RegularAccountUpdatableCode,
    }
}

pub use def::AccountType;

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

impl From<NativeAccountType> for AccountType {
    fn from(value: NativeAccountType) -> Self {
        match value {
            NativeAccountType::FungibleFaucet => AccountType::FungibleFaucet,
            NativeAccountType::NonFungibleFaucet => AccountType::NonFungibleFaucet,
            NativeAccountType::RegularAccountImmutableCode => {
                AccountType::RegularAccountImmutableCode
            },
            NativeAccountType::RegularAccountUpdatableCode => {
                AccountType::RegularAccountUpdatableCode
            },
        }
    }
}
