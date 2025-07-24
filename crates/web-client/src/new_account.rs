use miden_client::{
    Felt,
    account::{AccountBuilder, AccountType},
    auth::AuthSecretKey,
    crypto::SecretKey as NativeSecretKey,
};
use miden_lib::account::{auth::AuthRpoFalcon512, faucets::BasicFungibleFaucet};
use miden_objects::asset::TokenSymbol;
use rand::RngCore;
use wasm_bindgen::prelude::*;

use super::models::{
    account::Account, account_storage_mode::AccountStorageMode, secret_key::SecretKey, word::Word,
};
use crate::{WebClient, helpers::generate_wallet, js_error_with_context};

#[wasm_bindgen]
impl WebClient {
    #[wasm_bindgen(js_name = "newWallet")]
    pub async fn new_wallet(
        &mut self,
        storage_mode: &AccountStorageMode,
        mutable: bool,
        init_seed: Option<Vec<u8>>,
    ) -> Result<Account, JsValue> {
        let keystore = self.keystore.clone();
        if let Some(client) = self.get_mut_inner() {
            let (new_account, account_seed, key_pair) =
                generate_wallet(storage_mode, mutable, init_seed).await?;

            client
                .add_account(&new_account, Some(account_seed.into()), false)
                .await
                .map_err(|err| js_error_with_context(err, "failed to insert new wallet"))?;

            keystore
                .expect("KeyStore should be initialized")
                .add_key(&AuthSecretKey::RpoFalcon512(key_pair))
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
    ) -> Result<Account, JsValue> {
        if non_fungible {
            return Err(JsValue::from_str("Non-fungible faucets are not supported yet"));
        }

        let keystore = self.keystore.clone();
        if let Some(client) = self.get_mut_inner() {
            let key_pair = NativeSecretKey::with_rng(client.rng());
            let pub_key = key_pair.public_key();

            let mut init_seed = [0u8; 32];
            client.rng().fill_bytes(&mut init_seed);

            let symbol =
                TokenSymbol::new(token_symbol).map_err(|e| JsValue::from_str(&e.to_string()))?;
            let max_supply = Felt::try_from(max_supply.to_le_bytes().as_slice())
                .expect("u64 can be safely converted to a field element");

            let (new_account, seed) = match AccountBuilder::new(init_seed)
                .account_type(AccountType::FungibleFaucet)
                .storage_mode(storage_mode.into())
                .with_auth_component(AuthRpoFalcon512::new(pub_key))
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
                .add_key(&AuthSecretKey::RpoFalcon512(key_pair))
                .await
                .map_err(|err| err.to_string())?;

            match client.add_account(&new_account, Some(seed), false).await {
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
    pub async fn new_account(
        &mut self,
        account: &Account,
        account_seed: Option<Word>,
        overwrite: bool,
    ) -> Result<(), JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let account_seed = account_seed.map(Into::into);
            client
                .add_account(&account.into(), account_seed, overwrite)
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
        secret_key: &SecretKey,
    ) -> Result<(), JsValue> {
        let keystore = self.keystore.as_mut().expect("KeyStore should be initialized");
        keystore
            .add_key(&AuthSecretKey::RpoFalcon512(secret_key.into()))
            .await
            .map_err(|err| err.to_string())?;
        Ok(())
    }
}
