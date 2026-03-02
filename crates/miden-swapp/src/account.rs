use miden_crypto::utils::Deserializable;
use miden_mast_package::Package;
use miden_protocol::account::component::InitStorageData;
use miden_protocol::account::{AccountComponent, AccountType};
use miden_protocol::utils::sync::LazyLock;

// ACCOUNT COMPONENT
// ================================================================================================

const BASIC_WALLET_COMPONENT_BYTES: &[u8] =
    include_bytes!("../../../contracts/basic-wallet/basic_wallet.masp");

/// Initialize the basic-wallet account component only once by loading the embedded package.
static BASIC_WALLET_COMPONENT: LazyLock<AccountComponent> = LazyLock::new(|| {
    let package = Package::read_from_bytes(BASIC_WALLET_COMPONENT_BYTES)
        .expect("Failed to deserialize basic-wallet package");

    let init_storage_data = InitStorageData::default();

    AccountComponent::from_package(&package, &init_storage_data)
        .expect("Failed to create account component from basic-wallet package")
        .with_supported_types(From::from([
            AccountType::RegularAccountImmutableCode,
            AccountType::RegularAccountUpdatableCode,
        ]))
});

// BASIC WALLET
// ================================================================================================

/// SDK-side representation of the basic-wallet account component.
///
/// This mirrors `contracts/basic-wallet/` and provides helpers for loading
/// the compiled component and building accounts that use it.
pub struct BasicWallet;

impl BasicWallet {
    /// Returns the loaded basic-wallet account component.
    pub fn component() -> AccountComponent {
        BASIC_WALLET_COMPONENT.clone()
    }
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_wallet() {
        let _component = BasicWallet::component();
    }
}
