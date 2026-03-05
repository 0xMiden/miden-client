use miden_client::account::AccountFile as NativeAccountFile;
use js_export_macro::js_export;

use crate::models::account::Account;
use crate::models::account_id::AccountId;
use crate::platform::{JsBytes, JsErr};
use crate::utils::{deserialize_from_bytes, serialize_to_bytes};

#[derive(Debug, Clone)]
#[js_export]
pub struct AccountFile(NativeAccountFile);

#[js_export]
impl AccountFile {
    /// Returns the account ID.
    #[js_export(js_name = "accountId")]
    pub fn account_id(&self) -> AccountId {
        self.0.account.id().into()
    }

    /// Returns the account data.
    pub fn account(&self) -> Account {
        self.0.account.clone().into()
    }

    /// Returns the number of auth secret keys included.
    #[js_export(js_name = "authSecretKeyCount")]
    pub fn auth_secret_key_count(&self) -> usize {
        self.0.auth_secret_keys.len()
    }

    /// Serializes the `AccountFile` into a byte array
    pub fn serialize(&self) -> JsBytes {
        serialize_to_bytes(&self.0)
    }

    /// Deserializes a byte array into an `AccountFile`
    pub fn deserialize(bytes: JsBytes) -> Result<AccountFile, JsErr> {
        let native_account_file: NativeAccountFile = deserialize_from_bytes(&bytes)?;
        Ok(Self(native_account_file))
    }
}

impl From<NativeAccountFile> for AccountFile {
    fn from(native_account_file: NativeAccountFile) -> Self {
        Self(native_account_file)
    }
}

impl From<AccountFile> for NativeAccountFile {
    fn from(account_file: AccountFile) -> Self {
        account_file.0
    }
}

impl_napi_from_value!(AccountFile);
