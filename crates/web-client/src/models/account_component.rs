use miden_lib::account::auth::AuthRpoFalcon512 as NativeRpoFalcon512;
use miden_objects::account::{
    AccountComponent as NativeAccountComponent,
    StorageSlot as NativeStorageSlot,
};
use miden_objects::crypto::dsa::rpo_falcon512::SecretKey as NativeSecretKey;
use wasm_bindgen::prelude::*;

use crate::js_error_with_context;
use crate::models::assembler::Assembler;
use crate::models::secret_key::SecretKey;
use crate::models::storage_slot::StorageSlot;

#[wasm_bindgen]
pub struct AccountComponent(NativeAccountComponent);

#[wasm_bindgen]
impl AccountComponent {
    pub fn compile(
        account_code: &str,
        assembler: &Assembler,
        storage_slots: Vec<StorageSlot>,
    ) -> Result<AccountComponent, JsValue> {
        let native_slots: Vec<NativeStorageSlot> =
            storage_slots.into_iter().map(Into::into).collect();

        NativeAccountComponent::compile(account_code, assembler.into(), native_slots)
            .map(AccountComponent)
            .map_err(|e| js_error_with_context(e, "Failed to compile account component"))
    }

    #[wasm_bindgen(js_name = "withSupportsAllTypes")]
    pub fn with_supports_all_types(mut self) -> Self {
        self.0 = self.0.with_supports_all_types();
        self
    }

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

    #[wasm_bindgen(js_name = "createAuthComponent")]
    pub fn create_auth_component(secret_key: &SecretKey) -> AccountComponent {
        let native_secret_key: NativeSecretKey = secret_key.into();
        let native_auth_component: NativeAccountComponent =
            NativeRpoFalcon512::new(native_secret_key.public_key()).into();
        AccountComponent(native_auth_component)
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
