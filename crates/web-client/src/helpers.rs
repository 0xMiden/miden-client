use miden_client::account::component::BasicWallet;
use miden_client::account::{Account, AccountBuilder, AccountType};
use miden_client::auth::AuthRpoFalcon512;
use miden_client::crypto::rpo_falcon512::SecretKey;
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use wasm_bindgen::JsValue;

use crate::js_error_with_context;
use crate::models::account_storage_mode::AccountStorageMode;

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
) -> Result<(Account, SecretKey), JsValue> {
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

    let new_account = AccountBuilder::new(init_seed)
        .account_type(account_type)
        .storage_mode(storage_mode.into())
        .with_auth_component(AuthRpoFalcon512::new(key_pair.public_key()))
        .with_component(BasicWallet)
        .build()
        .map_err(|err| js_error_with_context(err, "failed to create new wallet"))?;

    let _account_seed =
        new_account.seed().expect("newly built wallet should always contain a seed");

    Ok((new_account, key_pair))
}
