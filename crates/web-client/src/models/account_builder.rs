use js_export_macro::js_export;
use miden_client::account::AccountBuilder as NativeAccountBuilder;
use miden_client::account::component::BasicWallet;
use miden_client::auth::NoAuth;

use crate::js_error_with_context;
use crate::models::account::Account;
use crate::models::account_component::AccountComponent;
use crate::models::account_storage_mode::AccountStorageMode;
use crate::models::account_type::AccountType;
use crate::models::word::Word;
use crate::platform::{JsErr, from_str_err};

#[js_export]
pub struct AccountBuilderResult {
    account: Account,
    seed: Word,
}

#[js_export]
impl AccountBuilderResult {
    /// Returns the built account.
    #[js_export(getter)]
    pub fn account(&self) -> Account {
        self.account.clone()
    }

    /// Returns the seed used to derive the account ID.
    #[js_export(getter)]
    pub fn seed(&self) -> Word {
        self.seed.clone()
    }
}

#[js_export]
#[derive(Clone)]
pub struct AccountBuilder(NativeAccountBuilder);

#[js_export]
impl AccountBuilder {
    /// Creates a new account builder from a 32-byte initial seed.
    #[js_export(constructor)]
    pub fn new(init_seed: Vec<u8>) -> Result<AccountBuilder, JsErr> {
        let seed_array: [u8; 32] = init_seed
            .try_into()
            .map_err(|_| from_str_err("Seed must be exactly 32 bytes"))?;
        Ok(AccountBuilder(NativeAccountBuilder::new(seed_array)))
    }

    /// Sets the account type (regular, faucet, etc.).
    #[js_export(js_name = "accountType")]
    pub fn account_type(&mut self, account_type: AccountType) -> Self {
        self.0 = self.0.clone().account_type(account_type.into());
        self.clone()
    }

    // TODO: AccountStorageMode as Enum
    /// Sets the storage mode (public/private) for the account.
    #[js_export(js_name = "storageMode")]
    pub fn storage_mode(&mut self, storage_mode: &AccountStorageMode) -> Self {
        self.0 = self.0.clone().storage_mode(storage_mode.into());
        self.clone()
    }

    /// Adds a component to the account.
    #[js_export(js_name = "withComponent")]
    pub fn with_component(&mut self, account_component: &AccountComponent) -> Self {
        self.0 = self.0.clone().with_component(account_component);
        self.clone()
    }

    /// Adds an authentication component to the account.
    #[js_export(js_name = "withAuthComponent")]
    pub fn with_auth_component(&mut self, account_component: &AccountComponent) -> Self {
        self.0 = self.0.clone().with_auth_component(account_component);
        self.clone()
    }

    /// Adds a no-auth component to the account (for public accounts).
    #[js_export(js_name = "withNoAuthComponent")]
    pub fn with_no_auth_component(&mut self) -> Self {
        self.0 = self.0.clone().with_auth_component(NoAuth);
        self.clone()
    }

    #[js_export(js_name = "withBasicWalletComponent")]
    pub fn with_basic_wallet_component(&mut self) -> Self {
        self.0 = self.0.clone().with_component(BasicWallet);
        self.clone()
    }

    /// Builds the account and returns it together with the derived seed.
    pub fn build(&self) -> Result<AccountBuilderResult, JsErr> {
        let account = self
            .0
            .clone()
            .build()
            .map_err(|err| js_error_with_context(err, "Failed to build account"))?;
        let seed = account.seed().expect("newly built account should always contain a seed");
        Ok(AccountBuilderResult {
            account: account.into(),
            seed: seed.into(),
        })
    }
}

