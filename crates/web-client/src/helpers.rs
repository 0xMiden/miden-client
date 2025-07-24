use miden_client::{
    account::{Account, AccountBuilder, AccountType},
    crypto::SecretKey,
};
use miden_lib::account::{auth::AuthRpoFalcon512, wallets::BasicWallet};
use miden_objects::Felt;
use rand::{RngCore, SeedableRng, rngs::StdRng};
use wasm_bindgen::JsValue;

use crate::{js_error_with_context, models::account_storage_mode::AccountStorageMode};

// HELPERS
// ================================================================================================
// These methods should not be exposed to the wasm interface

/// Serves as a way to manage the logic of seed generation.
///
/// # Errors:
/// - If rust client calls fail
/// - If the seed is passed in and is not exactly 32 bytes
pub(crate) async fn generate_wallet(
    storage_mode: &AccountStorageMode,
    mutable: bool,
    seed: Option<Vec<u8>>,
) -> Result<(Account, [Felt; 4], SecretKey), JsValue> {
    let mut rng = match seed {
        Some(seed_bytes) => {
            // Attempt to convert the seed slice into a 32-byte array.
            let seed_array: [u8; 32] = seed_bytes
                .try_into()
                .map_err(|_| JsValue::from_str("Seed must be exactly 32 bytes"))?;
            StdRng::from_seed(seed_array)
        },
        None => StdRng::from_os_rng(),
    };
    let key_pair = SecretKey::with_rng(&mut rng);
    let account_type = if mutable {
        AccountType::RegularAccountUpdatableCode
    } else {
        AccountType::RegularAccountImmutableCode
    };
    let mut init_seed = [0u8; 32];
    rng.fill_bytes(&mut init_seed);

    let (new_account, account_seed) = AccountBuilder::new(init_seed)
        .account_type(account_type)
        .storage_mode(storage_mode.into())
        .with_auth_component(AuthRpoFalcon512::new(key_pair.public_key()))
        .with_component(BasicWallet)
        .build()
        .map_err(|err| js_error_with_context(err, "failed to create new wallet"))?;

    Ok((new_account, *account_seed, key_pair))
}
