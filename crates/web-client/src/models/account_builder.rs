use miden_client::auth::NoAuth;
use miden_objects::account::AccountBuilder as NativeAccountBuilder;
use wasm_bindgen::prelude::*;

use crate::js_error_with_context;
use crate::models::account::Account;
use crate::models::account_component::AccountComponent;
use crate::models::account_storage_mode::AccountStorageMode;
use crate::models::account_type::AccountType;
use crate::models::word::Word;

#[wasm_bindgen]
pub struct AccountBuilderResult {
    account: Account,
    seed: Word,
}

#[wasm_bindgen]
impl AccountBuilderResult {
    #[wasm_bindgen(getter)]
    pub fn account(&self) -> Account {
        self.account.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn seed(&self) -> Word {
        self.seed.clone()
    }
}

#[wasm_bindgen]
pub struct AccountBuilder(NativeAccountBuilder);

#[wasm_bindgen]
impl AccountBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new(init_seed: Vec<u8>) -> Result<AccountBuilder, JsValue> {
        let seed_array: [u8; 32] = init_seed
            .try_into()
            .map_err(|_| JsValue::from_str("Seed must be exactly 32 bytes"))?;
        Ok(AccountBuilder(NativeAccountBuilder::new(seed_array)))
    }

    #[wasm_bindgen(js_name = "accountType")]
    pub fn account_type(mut self, account_type: AccountType) -> Self {
        self.0 = self.0.account_type(account_type.into());
        self
    }

    // TODO: AccountStorageMode as Enum
    #[wasm_bindgen(js_name = "storageMode")]
    pub fn storage_mode(mut self, storage_mode: &AccountStorageMode) -> Self {
        self.0 = self.0.storage_mode(storage_mode.into());
        self
    }

    #[wasm_bindgen(js_name = "withComponent")]
    pub fn with_component(mut self, account_component: &AccountComponent) -> Self {
        self.0 = self.0.with_component(account_component);
        self
    }

    #[wasm_bindgen(js_name = "withAuthComponent")]
    pub fn with_auth_component(mut self, account_component: &AccountComponent) -> Self {
        self.0 = self.0.with_auth_component(account_component);
        self
    }

    #[wasm_bindgen(js_name = "withNoAuthComponent")]
    pub fn with_no_auth_component(mut self) -> Self {
        self.0 = self.0.with_auth_component(NoAuth);
        self
    }

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
