use miden_client::account::AccountStorageMode as NativeAccountStorageMode;

#[napi(string_enum)]
pub enum AccountStorageMode {
    Private,
    Public,
    Network,
}

impl From<AccountStorageMode> for NativeAccountStorageMode {
    fn from(mode: AccountStorageMode) -> Self {
        match mode {
            AccountStorageMode::Private => NativeAccountStorageMode::Private,
            AccountStorageMode::Public => NativeAccountStorageMode::Public,
            AccountStorageMode::Network => NativeAccountStorageMode::Network,
        }
    }
}

impl From<&AccountStorageMode> for NativeAccountStorageMode {
    fn from(mode: &AccountStorageMode) -> Self {
        match mode {
            AccountStorageMode::Private => NativeAccountStorageMode::Private,
            AccountStorageMode::Public => NativeAccountStorageMode::Public,
            AccountStorageMode::Network => NativeAccountStorageMode::Network,
        }
    }
}
