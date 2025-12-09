use miden_client::Word as NativeWord;
use miden_client::account::StorageSlot as NativeStorageSlot;
use miden_client::account::component::AccountComponent as NativeAccountComponent;
use miden_client::auth::{
    AuthEcdsaK256Keccak as NativeEcdsaK256Keccak,
    AuthRpoFalcon512 as NativeRpoFalcon512,
    AuthSecretKey as NativeSecretKey,
    PublicKeyCommitment,
};
use miden_client::vm::Package as NativePackage;
use miden_core::mast::MastNodeExt;
use wasm_bindgen::prelude::*;

use crate::js_error_with_context;
use crate::models::auth::AuthScheme;
use crate::models::miden_arrays::StorageSlotArray;
use crate::models::package::Package;
use crate::models::script_builder::ScriptBuilder;
use crate::models::secret_key::SecretKey;
use crate::models::storage_slot::StorageSlot;
use crate::models::word::Word;

/// Procedure digest paired with whether it is an auth procedure.
#[derive(Clone)]
#[wasm_bindgen]
pub struct GetProceduresResultItem {
    digest: Word,
    is_auth: bool,
}

#[wasm_bindgen]
impl GetProceduresResultItem {
    /// Returns the MAST root digest for the procedure.
    #[wasm_bindgen(getter)]
    pub fn digest(&self) -> Word {
        self.digest.clone()
    }

    /// Returns true if the procedure is used for authentication.
    #[wasm_bindgen(getter, js_name = "isAuth")]
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

#[wasm_bindgen]
#[derive(Clone)]
pub struct AccountComponent(NativeAccountComponent);

#[wasm_bindgen]
impl AccountComponent {
    /// Compiles account code with the given storage slots using the provided assembler.
    pub fn compile(
        account_code: &str,
        builder: &ScriptBuilder,
        storage_slots: Vec<StorageSlot>,
    ) -> Result<AccountComponent, JsValue> {
        let native_slots: Vec<NativeStorageSlot> =
            storage_slots.into_iter().map(Into::into).collect();

        NativeAccountComponent::compile(account_code, builder.clone_assembler(), native_slots)
            .map(AccountComponent)
            .map_err(|e| js_error_with_context(e, "Failed to compile account component"))
    }

    /// Marks the component as supporting all account types.
    #[wasm_bindgen(js_name = "withSupportsAllTypes")]
    pub fn with_supports_all_types(mut self) -> Self {
        self.0 = self.0.with_supports_all_types();
        self
    }

    /// Returns the hex-encoded MAST root for a procedure by name.
    #[wasm_bindgen(js_name = "getProcedureHash")]
    pub fn get_procedure_hash(&self, procedure_name: &str) -> Result<String, JsValue> {
        let get_proc_export = self
            .0
            .library()
            .exports()
            .find(|export| export.name.name.as_str() == procedure_name)
            .ok_or_else(|| {
                JsValue::from_str(&format!(
                    "Procedure {procedure_name} not found in the account component library"
                ))
            })?;

        let get_proc_mast_id = self.0.library().get_export_node_id(&get_proc_export.name);

        let digest_hex = self
            .0
            .library()
            .mast_forest()
            .get_node_by_id(get_proc_mast_id)
            .ok_or_else(|| {
                JsValue::from_str(&format!("Mast node for procedure {procedure_name} not found",))
            })?
            .digest()
            .to_hex();

        Ok(digest_hex)
    }

    /// Returns all procedures exported by this component.
    #[wasm_bindgen(js_name = "getProcedures")]
    pub fn get_procedures(&self) -> Vec<GetProceduresResultItem> {
        self.0.get_procedures().iter().map(Into::into).collect()
    }

    fn create_auth_component(
        commitment: PublicKeyCommitment,
        auth_scheme: AuthScheme,
    ) -> Result<AccountComponent, JsValue> {
        match auth_scheme {
            AuthScheme::AuthRpoFalcon512 => {
                let auth = NativeRpoFalcon512::new(commitment);
                Ok(AccountComponent(auth.into()))
            },
            AuthScheme::AuthEcdsaK256Keccak => {
                let auth = NativeEcdsaK256Keccak::new(commitment);
                Ok(AccountComponent(auth.into()))
            },
        }
    }

    /// Builds an auth component from a secret key (`RpoFalcon512` or ECDSA k256 Keccak).
    #[wasm_bindgen(js_name = "createAuthComponentFromSecretKey")]
    pub fn create_auth_component_from_secret_key(
        secret_key: &SecretKey,
    ) -> Result<AccountComponent, JsValue> {
        let native_secret_key: NativeSecretKey = secret_key.into();
        let commitment = native_secret_key.public_key().to_commitment();

        let auth_scheme = match native_secret_key {
            NativeSecretKey::EcdsaK256Keccak(_) => AuthScheme::AuthEcdsaK256Keccak,
            NativeSecretKey::RpoFalcon512(_) => AuthScheme::AuthRpoFalcon512,
            // This is because the definition of NativeSecretKey has the
            // '#[non_exhaustive]' attribute, without this catch-all clause,
            // this is a compiler error.
            _unimplemented => {
                return Err(JsValue::from_str(
                    "building auth component for this auth scheme is not supported yet",
                ));
            },
        };

        AccountComponent::create_auth_component(commitment, auth_scheme)
    }

    #[wasm_bindgen(js_name = "createAuthComponentFromCommitment")]
    pub fn create_auth_component_from_commitment(
        commitment: &Word,
        auth_scheme: AuthScheme,
    ) -> Result<AccountComponent, JsValue> {
        let native_word: NativeWord = commitment.into();
        let pkc = PublicKeyCommitment::from(native_word);

        AccountComponent::create_auth_component(pkc, auth_scheme)
    }

    /// Creates an account component from a compiled package and storage slots.
    #[wasm_bindgen(js_name = "fromPackage")]
    pub fn from_package(
        package: &Package,
        storage_slots: &StorageSlotArray,
    ) -> Result<AccountComponent, JsValue> {
        let native_package: NativePackage = package.into();
        let native_library = native_package.unwrap_library().as_ref().clone();
        let native_slots: Vec<NativeStorageSlot> = storage_slots
            .__inner
            .iter()
            .map(|storage_slot| storage_slot.clone().into())
            .collect();

        NativeAccountComponent::new(native_library, native_slots)
            .map(AccountComponent)
            .map_err(|e| {
                js_error_with_context(e, "Failed to create account component from package")
            })
    }
}

// CONVERSIONS
// ================================================================================================

impl From<AccountComponent> for NativeAccountComponent {
    fn from(account_component: AccountComponent) -> Self {
        account_component.0
    }
}

impl From<&AccountComponent> for NativeAccountComponent {
    fn from(account_component: &AccountComponent) -> Self {
        account_component.0.clone()
    }
}
