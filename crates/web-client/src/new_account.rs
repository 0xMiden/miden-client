use miden_client::Felt;
use miden_client::account::component::BasicFungibleFaucet;
use miden_client::account::{AccountBuilder, AccountComponent, AccountType};
use miden_client::asset::TokenSymbol;
use miden_client::auth::{
    AuthEcdsaK256Keccak, AuthFalcon512Rpo, AuthSchemeId as NativeAuthScheme, AuthSecretKey,
};
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

use crate::prelude::*;
use super::models::account::Account;
use super::models::account_storage_mode::AccountStorageMode;
use super::models::auth::AuthScheme;
use crate::helpers::generate_wallet;
use crate::models::account_id::AccountId;
use crate::models::auth_secret_key::AuthSecretKey as BindingsAuthSecretKey;
use crate::WebClient;

// Shared methods
#[bindings]
impl WebClient {
    #[bindings(js_name = "newAccount")]
    pub async fn new_account(&self, account: &Account, overwrite: bool) -> platform::JsResult<()> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        let native_account = account.into();

        client
            .add_account(&native_account, overwrite)
            .await
            .map_err(|err| platform::error_with_context(err, "failed to insert new account"))?;
        Ok(())
    }

    #[bindings(js_name = "newWallet")]
    pub async fn new_wallet(
        &self,
        storage_mode: &AccountStorageMode,
        mutable: bool,
        auth_scheme: AuthScheme,
        init_seed: Option<Vec<u8>>,
    ) -> platform::JsResult<Account> {
        let (new_account, key_pair) =
            generate_wallet(storage_mode, mutable, init_seed, auth_scheme)?;

        // Add key to keystore
        {
            let keystore = lock_keystore!(self)?;
            #[cfg(feature = "wasm")]
            keystore
                .add_secret_key(&key_pair)
                .await
                .map_err(|err| {
                    platform::error_with_context(err, "failed to add secret key to keystore")
                })?;
            #[cfg(feature = "napi")]
            keystore
                .add_key(&key_pair)
                .map_err(|err| {
                    platform::error_with_context(err, "failed to add key to keystore")
                })?;
        }

        // Add account and register public keys
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        client
            .add_account(&new_account, false)
            .await
            .map_err(|err| platform::error_with_context(err, "failed to insert new wallet"))?;

        client
            .register_account_public_key_commitments(
                &new_account.id(),
                &[key_pair.public_key()],
            )
            .await
            .map_err(|err| {
                platform::error_with_context(err, "failed to map account to public keys")
            })?;

        Ok(new_account.into())
    }

    #[bindings(js_name = "newFaucet")]
    pub async fn new_faucet(
        &self,
        storage_mode: &AccountStorageMode,
        non_fungible: bool,
        token_symbol: String,
        decimals: u8,
        max_supply: i64,
        auth_scheme: AuthScheme,
    ) -> platform::JsResult<Account> {
        if non_fungible {
            return Err(platform::error_from_string(
                "Non-fungible faucets are not supported yet",
            ));
        }

        // Build the faucet account (needs client rng)
        let (new_account, key_pair) = {
            let mut guard = lock_client!(self);
            let client = guard
                .as_mut()
                .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

            let mut seed = [0u8; 32];
            client.rng().fill_bytes(&mut seed);
            // TODO: we need a way to pass the client's rng instead of having to use an stdrng
            let mut faucet_rng = StdRng::from_seed(seed);

            let native_scheme: NativeAuthScheme = auth_scheme.try_into()?;
            let (key_pair, auth_component) = match native_scheme {
                NativeAuthScheme::Falcon512Rpo => {
                    let key_pair = AuthSecretKey::new_falcon512_rpo_with_rng(&mut faucet_rng);
                    let auth_component: AccountComponent =
                        AuthFalcon512Rpo::new(key_pair.public_key().to_commitment()).into();
                    (key_pair, auth_component)
                },
                NativeAuthScheme::EcdsaK256Keccak => {
                    let key_pair = AuthSecretKey::new_ecdsa_k256_keccak_with_rng(&mut faucet_rng);
                    let auth_component: AccountComponent =
                        AuthEcdsaK256Keccak::new(key_pair.public_key().to_commitment()).into();
                    (key_pair, auth_component)
                },
                _ => {
                    let message = format!("unsupported auth scheme: {native_scheme:?}");
                    return Err(platform::error_from_string(&message));
                },
            };

            let max_supply_u64 = max_supply as u64;
            let symbol = TokenSymbol::new(&token_symbol)
                .map_err(|e| platform::error_with_context(e, "failed to create token symbol"))?;
            let max_supply_felt = Felt::try_from(max_supply_u64.to_le_bytes().as_slice())
                .expect("u64 can be safely converted to a field element");

            let mut init_seed = [0u8; 32];
            faucet_rng.fill_bytes(&mut init_seed);

            let new_account = AccountBuilder::new(init_seed)
                .account_type(AccountType::FungibleFaucet)
                .storage_mode(storage_mode.into())
                .with_auth_component(auth_component)
                .with_component(
                    BasicFungibleFaucet::new(symbol, decimals, max_supply_felt)
                        .map_err(|err| {
                            platform::error_with_context(err, "failed to create new faucet")
                        })?,
                )
                .build()
                .map_err(|err| {
                    let error_message = format!("Failed to create new faucet: {err:?}");
                    platform::error_from_string(&error_message)
                })?;

            (new_account, key_pair)
        }; // Drop client guard

        // Add key to keystore
        {
            let keystore = lock_keystore!(self)?;
            #[cfg(feature = "wasm")]
            keystore
                .add_secret_key(&key_pair)
                .await
                .map_err(|err| {
                    platform::error_with_context(err, "failed to add secret key to keystore")
                })?;
            #[cfg(feature = "napi")]
            keystore
                .add_key(&key_pair)
                .map_err(|err| {
                    platform::error_with_context(err, "failed to add key to keystore")
                })?;
        }

        // Re-lock client for account operations
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        client
            .register_account_public_key_commitments(
                &new_account.id(),
                &[key_pair.public_key()],
            )
            .await
            .map_err(|err| {
                platform::error_with_context(err, "failed to map account to public keys")
            })?;

        client
            .add_account(&new_account, false)
            .await
            .map(|_| new_account.into())
            .map_err(|err| {
                let error_message = format!("Failed to insert new faucet: {err:?}");
                platform::error_from_string(&error_message)
            })
    }
}

// wasm-only: add_account_secret_key_to_web_store (different method name + async keystore)
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WebClient {
    #[wasm_bindgen(js_name = "addAccountSecretKeyToWebStore")]
    pub async fn add_account_secret_key_to_web_store(
        &self,
        account_id: &AccountId,
        secret_key: &BindingsAuthSecretKey,
    ) -> platform::JsResult<()> {
        let keystore = lock_keystore!(self)?;
        let native_secret_key: AuthSecretKey = secret_key.into();
        let native_account_id = account_id.into();

        keystore
            .add_secret_key(&native_secret_key)
            .await
            .map_err(|err| {
                platform::error_with_context(err, "failed to add secret key to keystore")
            })?;

        let mut guard = lock_client!(self);
        if let Some(client) = guard.as_mut() {
            client
                .register_account_public_key_commitments(
                    &native_account_id,
                    &[native_secret_key.public_key()],
                )
                .await
                .map_err(|err| {
                    platform::error_with_context(err, "failed to map account to public keys")
                })?;
        }

        Ok(())
    }
}

// napi-only: add_secret_key (different method name + sync keystore)
#[cfg(feature = "napi")]
#[napi_derive::napi]
impl WebClient {
    pub async fn add_secret_key(
        &self,
        account_id: &AccountId,
        secret_key: &BindingsAuthSecretKey,
    ) -> platform::JsResult<()> {
        let keystore = lock_keystore!(self)?;
        let native_secret_key: AuthSecretKey = secret_key.into();
        let native_account_id = account_id.into();

        keystore
            .add_key(&native_secret_key)
            .map_err(|err| platform::error_with_context(err, "failed to add key to keystore"))?;

        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        client
            .register_account_public_key_commitments(
                &native_account_id,
                &[native_secret_key.public_key()],
            )
            .await
            .map_err(|err| {
                platform::error_with_context(err, "failed to map account to public keys")
            })?;

        Ok(())
    }
}
