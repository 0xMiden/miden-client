use js_export_macro::js_export;
use miden_client::Felt;
use miden_client::account::component::BasicFungibleFaucet;
use miden_client::account::{AccountBuilder, AccountComponent, AccountType};
use miden_client::asset::TokenSymbol;
use miden_client::auth::{
    AuthEcdsaK256Keccak,
    AuthFalcon512Rpo,
    AuthSchemeId as NativeAuthScheme,
    AuthSecretKey,
};
use miden_client::block::BlockNumber;
use miden_client::keystore::Keystore;
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

use super::models::account::Account;
use super::models::account_storage_mode::AccountStorageMode;
use super::models::auth::AuthScheme;
use super::models::auth_secret_key::AuthSecretKey as WebAuthSecretKey;
use crate::helpers::generate_wallet;
use crate::models::account_id::AccountId;
use crate::platform::{JsErr, JsU64, from_str_err, maybe_wrap_send};
use crate::{WebClient, js_error_with_context};

impl WebClient {
    /// Syncs state if the client has never been synced (still at genesis block).
    ///
    /// This prevents a slow full-chain scan on the next sync after account creation.
    /// Errors are intentionally ignored — account creation should proceed regardless.
    async fn maybe_sync_before_account_creation(&self) {
        let should_sync = {
            let mut guard = self.get_mut_inner().await;
            match guard.as_mut() {
                Some(client) => client.get_sync_height().await.is_ok_and(|h| h == BlockNumber::GENESIS),
                None => false,
            }
        };

        if should_sync {
            let mut guard = self.get_mut_inner().await;
            if let Some(client) = guard.as_mut() {
                let _ = maybe_wrap_send(client.sync_state()).await;
            }
        }
    }
}

#[js_export]
impl WebClient {
    #[js_export(js_name = "newFaucet")]
    pub async fn new_faucet(
        &self,
        storage_mode: &AccountStorageMode,
        non_fungible: bool,
        token_symbol: String,
        decimals: u8,
        max_supply: JsU64,
        auth_scheme: AuthScheme,
    ) -> Result<Account, JsErr> {
        let max_supply = max_supply as u64;
        self.maybe_sync_before_account_creation().await;
        if non_fungible {
            return Err(from_str_err("Non-fungible faucets are not supported yet"));
        }

        let keystore = self.keystore().await?;

        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;

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
                return Err(from_str_err(&message));
            },
        };

        let symbol =
            TokenSymbol::new(&token_symbol).map_err(|e| from_str_err(&e.to_string()))?;
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
                return Err(from_str_err(&error_message));
            },
        };

        keystore
            .add_key(&key_pair, new_account.id())
            .await
            .map_err(|err| from_str_err(&err.to_string()))?;

        match client.add_account(&new_account, false).await {
            Ok(_) => Ok(new_account.into()),
            Err(err) => {
                let error_message = format!("Failed to insert new faucet: {err:?}");
                Err(from_str_err(&error_message))
            },
        }
    }

    #[js_export(js_name = "newWallet")]
    pub async fn new_wallet(
        &self,
        storage_mode: &AccountStorageMode,
        mutable: bool,
        auth_scheme: AuthScheme,
        init_seed: Option<Vec<u8>>,
    ) -> Result<Account, JsErr> {
        self.maybe_sync_before_account_creation().await;
        let keystore = self.keystore().await?;

        let (new_account, key_pair) =
            generate_wallet(storage_mode, mutable, init_seed, auth_scheme).await?;

        {
            let mut guard = self.get_mut_inner().await;
            let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
            client
                .add_account(&new_account, false)
                .await
                .map_err(|err| js_error_with_context(err, "failed to insert new wallet"))?;
        }

        keystore
            .add_key(&key_pair, new_account.id())
            .await
            .map_err(|err| from_str_err(&err.to_string()))?;

        Ok(new_account.into())
    }

    #[js_export(js_name = "newAccount")]
    pub async fn new_account(&self, account: &Account, overwrite: bool) -> Result<(), JsErr> {
        self.maybe_sync_before_account_creation().await;
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        let native_account = account.into();

        client
            .add_account(&native_account, overwrite)
            .await
            .map_err(|err| js_error_with_context(err, "failed to insert new account"))?;
        Ok(())
    }

    #[js_export(js_name = "addAccountSecretKeyToWebStore")]
    pub async fn add_account_secret_key_to_web_store(
        &self,
        account_id: &AccountId,
        secret_key: &WebAuthSecretKey,
    ) -> Result<(), JsErr> {
        let keystore = self.keystore().await?;
        let native_secret_key: AuthSecretKey = secret_key.into();
        let native_account_id = account_id.into();

        keystore
            .add_key(&native_secret_key, native_account_id)
            .await
            .map_err(|err| from_str_err(&err.to_string()))?;

        Ok(())
    }
}
