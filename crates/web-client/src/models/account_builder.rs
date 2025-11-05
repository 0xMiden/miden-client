use miden_client::account::AccountBuilder as NativeAccountBuilder;
use miden_client::auth::NoAuth;
use wasm_bindgen::prelude::*;

use crate::js_error_with_context;
use crate::models::account::Account;
use crate::models::account_component::AccountComponent;
use crate::models::account_storage_mode::AccountStorageMode;
use crate::models::account_type::AccountType;
use crate::models::word::Word;

/// Result of constructing a new account via [`AccountBuilder`].
///
/// Exposes the built account and the seed used to derive it so values can be persisted on the
/// JavaScript side.
#[wasm_bindgen]
pub struct AccountBuilderResult {
    account: Account,
    seed: Word,
}

#[wasm_bindgen]
impl AccountBuilderResult {
    /// Returns the newly built account instance.
    #[wasm_bindgen(getter)]
    pub fn account(&self) -> Account {
        self.account.clone()
    }

    /// Returns the seed used to derive the account keys.
    #[wasm_bindgen(getter)]
    pub fn seed(&self) -> Word {
        self.seed.clone()
    }
}

/// JavaScript wrapper around [`miden_client::account::AccountBuilder`].
///
/// Provides a builder interface for configuring and creating new accounts in the browser.
#[wasm_bindgen]
pub struct AccountBuilder(NativeAccountBuilder);

#[wasm_bindgen]
impl AccountBuilder {
    /// Creates a new account builder from a 32-byte seed.
    ///
    /// @param init_seed - Seed bytes; must be exactly 32 bytes.
    /// @throws Throws if the seed length is invalid.
    #[wasm_bindgen(constructor)]
    pub fn new(init_seed: Vec<u8>) -> Result<AccountBuilder, JsValue> {
        let seed_array: [u8; 32] = init_seed
            .try_into()
            .map_err(|_| JsValue::from_str("Seed must be exactly 32 bytes"))?;
        Ok(AccountBuilder(NativeAccountBuilder::new(seed_array)))
    }

    #[wasm_bindgen(js_name = "accountType")]
    /// Sets the account type for the builder.
    pub fn account_type(mut self, account_type: AccountType) -> Self {
        self.0 = self.0.account_type(account_type.into());
        self
    }

    // TODO: AccountStorageMode as Enum
    #[wasm_bindgen(js_name = "storageMode")]
    /// Sets the account storage mode (e.g. private or network).
    pub fn storage_mode(mut self, storage_mode: &AccountStorageMode) -> Self {
        self.0 = self.0.storage_mode(storage_mode.into());
        self
    }

    #[wasm_bindgen(js_name = "withComponent")]
    /// Adds an additional account component to the builder.
    pub fn with_component(mut self, account_component: &AccountComponent) -> Self {
        self.0 = self.0.with_component(account_component);
        self
    }

    #[wasm_bindgen(js_name = "withAuthComponent")]
    /// Sets the authentication component to use for the account.
    pub fn with_auth_component(mut self, account_component: &AccountComponent) -> Self {
        self.0 = self.0.with_auth_component(account_component);
        self
    }

    #[wasm_bindgen(js_name = "withNoAuthComponent")]
    /// Configures the account to use the built-in no-auth component.
    pub fn with_no_auth_component(mut self) -> Self {
        self.0 = self.0.with_auth_component(NoAuth);
        self
    }

    /// Builds the account using the accumulated configuration.
    ///
    /// @throws Throws if the underlying account creation fails.
    pub fn build(self) -> Result<AccountBuilderResult, JsValue> {
        let account = self
            .0
            .build()
            .map_err(|err| js_error_with_context(err, "Failed to build account"))?;
        let seed = account.seed().expect("newly built account should always contain a seed");
        Ok(AccountBuilderResult {
            account: account.into(),
            seed: seed.into(),
        })
    }
}
