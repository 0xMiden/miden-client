use js_export_macro::js_export;
use miden_client::account::AccountType as NativeAccountType;

#[derive(Clone)]
#[js_export]
pub enum AccountType {
    FungibleFaucet,
    NonFungibleFaucet,
    RegularAccountImmutableCode,
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
