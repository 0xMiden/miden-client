use miden_client::Felt;
use miden_client::account::component::BasicFungibleFaucet;
use miden_client::account::{AccountBuilder, AccountComponent, AccountType};
use miden_client::asset::TokenSymbol;
use miden_client::auth::{AuthEcdsaK256Keccak, AuthRpoFalcon512, AuthSecretKey};
use miden_objects::account::auth::AuthScheme as NativeAuthScheme;
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use wasm_bindgen::prelude::*;

use super::models::account::Account;
use super::models::account_storage_mode::AccountStorageMode;
use super::models::auth::AuthScheme;
use super::models::auth_secret_key::AuthSecretKey as WebAuthSecretKey;
use crate::helpers::generate_wallet;
use crate::{WebClient, js_error_with_context};

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
        let keystore = self.keystore.clone();
        if let Some(client) = self.get_mut_inner() {
            let (new_account, key_pair) =
                generate_wallet(storage_mode, mutable, init_seed, auth_scheme).await?;

            client
                .add_account(&new_account, false)
                .await
                .map_err(|err| js_error_with_context(err, "failed to insert new wallet"))?;

            keystore
                .expect("KeyStore should be initialized")
                .add_key(&key_pair)
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
        if non_fungible {
            return Err(JsValue::from_str("Non-fungible faucets are not supported yet"));
        }

        let keystore = self.keystore.clone();
        if let Some(client) = self.get_mut_inner() {
            let mut seed = [0u8; 32];
            client.rng().fill_bytes(&mut seed);
            // TODO: we need a way to pass the client's rng instead of having to use an stdrng
            let mut faucet_rng = StdRng::from_seed(seed);

            let native_scheme: NativeAuthScheme = auth_scheme.try_into()?;
            let (key_pair, auth_component) = match native_scheme {
                NativeAuthScheme::RpoFalcon512 => {
                    let key_pair = AuthSecretKey::new_rpo_falcon512_with_rng(&mut faucet_rng);
                    let auth_component: AccountComponent =
                        AuthRpoFalcon512::new(key_pair.public_key().to_commitment()).into();
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
                    return Err(JsValue::from_str(&message));
                },
            };

            let symbol =
                TokenSymbol::new(token_symbol).map_err(|e| JsValue::from_str(&e.to_string()))?;
            let max_supply = Felt::try_from(max_supply.to_le_bytes().as_slice())
                .expect("u64 can be safely converted to a field element");

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
                .build()
            {
                Ok(result) => result,
                Err(err) => {
                    let error_message = format!("Failed to create new faucet: {err:?}");
                    return Err(JsValue::from_str(&error_message));
                },
            };

            keystore
                .expect("KeyStore should be initialized")
                .add_key(&key_pair)
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

    #[wasm_bindgen(js_name = "addAccountSecretKeyToWebStore")]
    pub async fn add_account_secret_key_to_web_store(
        &mut self,
        secret_key: &WebAuthSecretKey,
    ) -> Result<(), JsValue> {
        let keystore = self.keystore.as_mut().expect("KeyStore should be initialized");
        keystore.add_key(secret_key.into()).await.map_err(|err| err.to_string())?;
        Ok(())
    }
}
