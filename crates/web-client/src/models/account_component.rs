use miden_client::Word as NativeWord;
use miden_client::account::component::AccountComponent as NativeAccountComponent;
use miden_client::account::{
    AccountComponentCode as NativeAccountComponentCode,
    StorageSlot as NativeStorageSlot,
};
use miden_client::assembly::{Library as NativeLibrary, MastNodeExt};
use miden_client::auth::{
    AuthEcdsaK256Keccak as NativeEcdsaK256Keccak,
    AuthFalcon512Rpo as NativeFalcon512Rpo,
    AuthSecretKey as NativeSecretKey,
    PublicKeyCommitment,
};
use miden_client::vm::Package as NativePackage;
use js_export_macro::js_export;

use crate::js_error_with_context;
use crate::platform::{JsErr, from_str_err};
use crate::models::account_component_code::AccountComponentCode;
use crate::models::auth::AuthScheme;
use crate::models::auth_secret_key::AuthSecretKey;
use crate::models::library::Library;
use crate::models::miden_arrays::StorageSlotArray;
use crate::models::package::Package;
use crate::models::storage_slot::StorageSlot;
use crate::models::word::Word;

/// Procedure digest paired with whether it is an auth procedure.
#[derive(Clone)]
#[js_export]
pub struct GetProceduresResultItem {
    digest: Word,
    is_auth: bool,
}

#[js_export]
impl GetProceduresResultItem {
    /// Returns the MAST root digest for the procedure.
    #[js_export(getter)]
    pub fn digest(&self) -> Word {
        self.digest.clone()
    }

    /// Returns true if the procedure is used for authentication.
    #[js_export(getter, js_name = "isAuth")]
    pub fn is_auth(&self) -> bool {
        self.is_auth
    }
}

impl From<&(NativeWord, bool)> for GetProceduresResultItem {
    fn from(native_get_procedures_result_item: &(NativeWord, bool)) -> Self {
        Self {
            digest: native_get_procedures_result_item.0.into(),
            is_auth: native_get_procedures_result_item.1,
        }
    }
}

#[js_export]
#[derive(Clone)]
pub struct AccountComponent(NativeAccountComponent);

#[js_export]
impl AccountComponent {
    /// Compiles account code with the given storage slots using the provided assembler.
    pub fn compile(
        account_code: AccountComponentCode,
        storage_slots: Vec<StorageSlot>,
    ) -> Result<AccountComponent, JsErr> {
        let native_slots: Vec<NativeStorageSlot> =
            storage_slots.into_iter().map(Into::into).collect();

        let native_account_code: NativeAccountComponentCode = account_code.into();

        NativeAccountComponent::new(native_account_code, native_slots)
            .map(AccountComponent)
            .map_err(|e| js_error_with_context(e, "Failed to compile account component"))
    }

    /// Marks the component as supporting all account types.
    #[js_export(js_name = "withSupportsAllTypes")]
    pub fn with_supports_all_types(&mut self) -> Self {
        self.0 = self.0.clone().with_supports_all_types();
        self.clone()
    }

    /// Returns the hex-encoded MAST root for a procedure by name.
    #[js_export(js_name = "getProcedureHash")]
    pub fn get_procedure_hash(&self, procedure_name: String) -> Result<String, JsErr> {
        let library = self.0.component_code().as_library();

        let get_proc_export = library
            .exports()
            .find(|export| {
                export.as_procedure().is_some()
                    && (export.path().as_ref().as_str() == procedure_name.as_str()
                        || export.path().as_ref().to_relative().as_str() == procedure_name.as_str())
            })
            .ok_or_else(|| {
                from_str_err(&format!(
                    "Procedure {procedure_name} not found in the account component library"
                ))
            })?;

        let get_proc_mast_id = library.get_export_node_id(get_proc_export.path());

        let digest_hex = library
            .mast_forest()
            .get_node_by_id(get_proc_mast_id)
            .ok_or_else(|| {
                from_str_err(&format!("Mast node for procedure {procedure_name} not found"))
            })?
            .digest()
            .to_hex();

        Ok(digest_hex)
    }

    /// Returns all procedures exported by this component.
    #[js_export(js_name = "getProcedures")]
    pub fn get_procedures(&self) -> Vec<GetProceduresResultItem> {
        self.0.get_procedures().iter().map(Into::into).collect()
    }

    /// Builds an auth component from a secret key, inferring the auth scheme from the key type.
    #[js_export(js_name = "createAuthComponentFromSecretKey")]
    pub fn create_auth_component_from_secret_key(
        secret_key: &AuthSecretKey,
    ) -> Result<AccountComponent, JsErr> {
        let native_secret_key: NativeSecretKey = secret_key.into();
        let commitment = native_secret_key.public_key().to_commitment();

        let auth_scheme = match native_secret_key {
            NativeSecretKey::EcdsaK256Keccak(_) => AuthScheme::AuthEcdsaK256Keccak,
            NativeSecretKey::Falcon512Rpo(_) => AuthScheme::AuthRpoFalcon512,
            // This is because the definition of NativeSecretKey has the
            // '#[non_exhaustive]' attribute, without this catch-all clause,
            // this is a compiler error.
            _unimplemented => {
                return Err(from_str_err(
                    "building auth component for this auth scheme is not supported yet",
                ));
            },
        };

        Ok(AccountComponent::create_auth_component(commitment, auth_scheme))
    }

    #[js_export(js_name = "createAuthComponentFromCommitment")]
    pub fn create_auth_component_from_commitment(
        commitment: &Word,
        auth_scheme: AuthScheme,
    ) -> Result<AccountComponent, JsErr> {
        let native_word: NativeWord = commitment.into();
        let pkc = PublicKeyCommitment::from(native_word);

        Ok(AccountComponent::create_auth_component(pkc, auth_scheme))
    }

    /// Creates an account component from a compiled package and storage slots.
    #[js_export(js_name = "fromPackage")]
    pub fn from_package(
        package: &Package,
        storage_slots: StorageSlotArray,
    ) -> Result<AccountComponent, JsErr> {
        let native_package: NativePackage = package.into();
        let native_library = native_package.unwrap_library().as_ref().clone();
        #[cfg(feature = "browser")]
        let native_slots: Vec<NativeStorageSlot> = storage_slots
            .__inner
            .into_iter()
            .map(|storage_slot| storage_slot.into())
            .collect();
        #[cfg(feature = "nodejs")]
        let native_slots: Vec<NativeStorageSlot> = Vec::from(storage_slots)
            .into_iter()
            .map(|storage_slot| storage_slot.into())
            .collect();

        NativeAccountComponent::new(native_library, native_slots)
            .map(AccountComponent)
            .map_err(|e| {
                js_error_with_context(e, "Failed to create account component from package")
            })
    }

    /// Creates an account component from a compiled library and storage slots.
    #[js_export(js_name = "fromLibrary")]
    pub fn from_library(
        library: &Library,
        storage_slots: Vec<StorageSlot>,
    ) -> Result<AccountComponent, JsErr> {
        let native_library: NativeLibrary = library.into();
        let native_slots: Vec<NativeStorageSlot> =
            storage_slots.into_iter().map(Into::into).collect();

        NativeAccountComponent::new(native_library, native_slots)
            .map(AccountComponent)
            .map_err(|e| {
                js_error_with_context(e, "Failed to create account component from library")
            })
    }
}

impl AccountComponent {
    fn create_auth_component(
        commitment: PublicKeyCommitment,
        auth_scheme: AuthScheme,
    ) -> AccountComponent {
        match auth_scheme {
            AuthScheme::AuthRpoFalcon512 => {
                let auth = NativeFalcon512Rpo::new(commitment);
                AccountComponent(auth.into())
            },
            AuthScheme::AuthEcdsaK256Keccak => {
                let auth = NativeEcdsaK256Keccak::new(commitment);
                AccountComponent(auth.into())
            },
        }
    }
}

// CONVERSIONS
// ================================================================================================

impl From<AccountComponent> for NativeAccountComponent {
    fn from(account_component: AccountComponent) -> Self {
        account_component.0
    }
}

impl From<NativeAccountComponent> for AccountComponent {
    fn from(native_account_component: NativeAccountComponent) -> Self {
        AccountComponent(native_account_component)
    }
}

impl From<&AccountComponent> for NativeAccountComponent {
    fn from(account_component: &AccountComponent) -> Self {
        account_component.0.clone()
    }
}
