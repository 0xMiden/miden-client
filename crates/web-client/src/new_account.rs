use miden_client::Felt;
use miden_client::account::component::{AuthControlled, BasicFungibleFaucet};
use miden_client::account::{AccountBuilder, AccountComponent, AccountType};
use miden_client::asset::TokenSymbol;
use miden_client::auth::{AuthSchemeId as NativeAuthScheme, AuthSecretKey, AuthSingleSig};
use miden_client::block::BlockNumber;
use miden_client::keystore::Keystore;
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use wasm_bindgen::prelude::*;

use super::models::account::Account;
use super::models::account_storage_mode::AccountStorageMode;
use super::models::auth::AuthScheme;
use super::models::auth_secret_key::AuthSecretKey as WebAuthSecretKey;
use crate::helpers::generate_wallet;
use crate::{WebClient, js_error_with_context};

impl WebClient {
    /// Syncs state if the client has never been synced (still at genesis block).
    ///
    /// This prevents a slow full-chain scan on the next sync after account creation.
    /// Errors are intentionally ignored — account creation should proceed regardless.
    async fn maybe_sync_before_account_creation(&mut self) {
        let should_sync = match self.get_mut_inner() {
            Some(client) => client.get_sync_height().await.is_ok_and(|h| h == BlockNumber::GENESIS),
            None => false,
        };

        if should_sync && let Some(client) = self.get_mut_inner() {
            let _ = client.sync_state().await;
        }
    }
}

#[wasm_bindgen]
impl WebClient {
    #[wasm_bindgen(js_name = "newWallet")]
    pub async fn new_wallet(
        &mut self,
        storage_mode: &AccountStorageMode,
        mutable: bool,
        auth_scheme: AuthScheme,
        init_seed: Option<Vec<u8>>,
    ) -> Result<Account, JsValue> {
        self.maybe_sync_before_account_creation().await;
        let keystore = self.inner_keystore()?.clone();
        if let Some(client) = self.get_mut_inner() {
            let (new_account, key_pair) =
                generate_wallet(storage_mode, mutable, init_seed, auth_scheme).await?;

            client
                .add_account(&new_account, false)
                .await
                .map_err(|err| js_error_with_context(err, "failed to insert new wallet"))?;

            keystore
                .add_key(&key_pair, new_account.id())
                .await
                .map_err(|err| err.to_string())?;

            Ok(new_account.into())
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "newFaucet")]
    pub async fn new_faucet(
        &mut self,
        storage_mode: &AccountStorageMode,
        non_fungible: bool,
        token_symbol: &str,
        decimals: u8,
        max_supply: u64,
        auth_scheme: AuthScheme,
    ) -> Result<Account, JsValue> {
        self.maybe_sync_before_account_creation().await;
        if non_fungible {
            return Err(JsValue::from_str("Non-fungible faucets are not supported yet"));
        }

        let keystore = self.inner_keystore()?.clone();

        if let Some(client) = self.get_mut_inner() {
            let mut seed = [0u8; 32];
            client.rng().fill_bytes(&mut seed);
            // TODO: we need a way to pass the client's rng instead of having to use an stdrng
            let mut faucet_rng = StdRng::from_seed(seed);

            let native_scheme: NativeAuthScheme = auth_scheme.try_into()?;
            let key_pair = match native_scheme {
                NativeAuthScheme::Falcon512Poseidon2 => {
                    AuthSecretKey::new_falcon512_poseidon2_with_rng(&mut faucet_rng)
                },
                NativeAuthScheme::EcdsaK256Keccak => {
                    AuthSecretKey::new_ecdsa_k256_keccak_with_rng(&mut faucet_rng)
                },
                _ => {
                    let message = format!("unsupported auth scheme: {native_scheme:?}");
                    return Err(JsValue::from_str(&message));
                },
            };
            let auth_component: AccountComponent =
                AuthSingleSig::new(key_pair.public_key().to_commitment(), native_scheme).into();

            let symbol =
                TokenSymbol::new(token_symbol).map_err(|e| JsValue::from_str(&e.to_string()))?;
            let max_supply = Felt::new(max_supply);

            let mut init_seed = [0u8; 32];
            faucet_rng.fill_bytes(&mut init_seed);

            let new_account = match AccountBuilder::new(init_seed)
                .account_type(AccountType::FungibleFaucet)
                .storage_mode(storage_mode.into())
                .with_auth_component(auth_component)
                .with_component(
                    BasicFungibleFaucet::new(symbol, decimals, max_supply)
                        .map_err(|err| js_error_with_context(err, "failed to create new faucet"))?,
                )
                .with_component(AuthControlled::allow_all())
                .build()
            {
                Ok(result) => result,
                Err(err) => {
                    let error_message = format!("Failed to create new faucet: {err:?}");
                    return Err(JsValue::from_str(&error_message));
                },
            };

            keystore
                .add_key(&key_pair, new_account.id())
                .await
                .map_err(|err| err.to_string())?;

            match client.add_account(&new_account, false).await {
                Ok(_) => Ok(new_account.into()),
                Err(err) => {
                    let error_message = format!("Failed to insert new faucet: {err:?}");
                    Err(JsValue::from_str(&error_message))
                },
            }
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "newAccount")]
    pub async fn new_account(&mut self, account: &Account, overwrite: bool) -> Result<(), JsValue> {
        self.maybe_sync_before_account_creation().await;
        if let Some(client) = self.get_mut_inner() {
            let native_account = account.into();

            client
                .add_account(&native_account, overwrite)
                .await
                .map_err(|err| js_error_with_context(err, "failed to insert new account"))?;
            Ok(())
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    /// Inserts an account and its secret key in one call, matching how
    /// `newWallet` / `newFaucet` already work internally.  If the key
    /// insertion fails the account is still persisted (same as wallet/faucet),
    /// but callers only need a single await instead of two.
    #[wasm_bindgen(js_name = "newAccountWithSecretKey")]
    pub async fn new_account_with_secret_key(
        &mut self,
        account: &Account,
        secret_key: &WebAuthSecretKey,
    ) -> Result<(), JsValue> {
        self.maybe_sync_before_account_creation().await;
        if let Some(client) = self.get_mut_inner() {
            let native_account: miden_client::account::Account = account.into();
            let account_id = native_account.id();

            client
                .add_account(&native_account, false)
                .await
                .map_err(|err| js_error_with_context(err, "failed to insert new account"))?;

            let keystore = self.inner_keystore()?;
            let native_secret_key: AuthSecretKey = secret_key.into();

            keystore
                .add_key(&native_secret_key, account_id)
                .await
                .map_err(|err| js_error_with_context(err, "failed to add secret key"))?;

            Ok(())
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }
}
