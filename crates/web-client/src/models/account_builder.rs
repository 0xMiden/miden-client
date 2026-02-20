use miden_client::account::AccountBuilder as NativeAccountBuilder;
use miden_client::account::component::BasicWallet;
use miden_client::auth::NoAuth;
use crate::prelude::*;

use crate::models::account::Account;
use crate::models::account_component::AccountComponent;
use crate::models::account_storage_mode::AccountStorageMode;
use crate::models::account_type::AccountType;
use crate::models::word::Word;

/// Result of building an account: the account itself and the seed used.
#[bindings]
pub struct AccountBuilderResult {
    account: Account,
    seed: Word,
}

#[bindings]
impl AccountBuilderResult {
    /// Returns the built account.
    #[bindings(getter)]
    pub fn account(&self) -> Account {
        self.account.clone()
    }

    /// Returns the seed used to derive the account ID.
    #[bindings(getter)]
    pub fn seed(&self) -> Word {
        self.seed.clone()
    }
}

#[bindings]
pub struct AccountBuilder(Option<NativeAccountBuilder>);

impl AccountBuilder {
    fn take_inner(&mut self) -> NativeAccountBuilder {
        self.0.take().expect("AccountBuilder has already been consumed by build()")
    }
}

#[bindings]
impl AccountBuilder {
    /// Creates a new account builder from a 32-byte initial seed.
    #[bindings(constructor)]
    pub fn new(init_seed: Vec<u8>) -> JsResult<AccountBuilder> {
        let seed_array: [u8; 32] = init_seed
            .try_into()
            .map_err(|_| platform::error_from_string("Seed must be exactly 32 bytes"))?;
        Ok(AccountBuilder(Some(NativeAccountBuilder::new(seed_array))))
    }

    /// Sets the account type (regular, faucet, etc.).
    pub fn account_type(&mut self, account_type: AccountType) {
        let inner = self.take_inner();
        self.0 = Some(inner.account_type(account_type.into()));
    }

    /// Sets the storage mode (public/private) for the account.
    pub fn storage_mode(&mut self, storage_mode: &AccountStorageMode) {
        let inner = self.take_inner();
        self.0 = Some(inner.storage_mode(storage_mode.into()));
    }

    /// Adds a component to the account.
    pub fn with_component(&mut self, account_component: &AccountComponent) {
        let inner = self.take_inner();
        self.0 = Some(inner.with_component(account_component));
    }

    /// Adds an authentication component to the account.
    pub fn with_auth_component(&mut self, account_component: &AccountComponent) {
        let inner = self.take_inner();
        self.0 = Some(inner.with_auth_component(account_component));
    }

    /// Adds a no-auth component to the account (for public accounts).
    pub fn with_no_auth_component(&mut self) {
        let inner = self.take_inner();
        self.0 = Some(inner.with_auth_component(NoAuth));
    }

    /// Adds a basic wallet component to the account.
    pub fn with_basic_wallet_component(&mut self) {
        let inner = self.take_inner();
        self.0 = Some(inner.with_component(BasicWallet));
    }

    /// Builds the account and returns it together with the derived seed.
    pub fn build(&mut self) -> JsResult<AccountBuilderResult> {
        let inner = self.take_inner();
        let account = inner
            .build()
            .map_err(|err| platform::error_with_context(err, "Failed to build account"))?;
        let seed = account.seed().expect("newly built account should always contain a seed");
        Ok(AccountBuilderResult {
            account: account.into(),
            seed: seed.into(),
        })
    }
}
