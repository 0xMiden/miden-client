use miden_objects::account::{
    code::procedure, AccountComponent as NativeAccountComponent, StorageSlot as NativeStorageSlot
};
use wasm_bindgen::prelude::*;

use crate::{
    js_error_with_context,
    models::{assembler::Assembler, library::Library, storage_slot::StorageSlot},
};

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

    pub fn library(&self) -> Library {
        self.0.library().clone().into()
    }

    #[wasm_bindgen(js_name = "withSupportsAllTypes")]
    pub fn with_supports_all_types(mut self) -> Self {
        self.0 = self.0.with_supports_all_types();
        self
    }

    #[wasm_bindgen(js_name = "getProcedureHash")]
    pub fn get_procedure_hash(&self, procedure_name: &str) -> Result<String, JsValue> {
        let get_proc_export = self.0
        .library()
        .exports()
        .find(|export| export.name.as_str() == procedure_name)
        .ok_or_else(|| {
            JsValue::from_str(&format!(
                "Procedure '{}' not found in the account component library",
                procedure_name
            ))
        })?;

        let get_proc_mast_id = self.0
            .library()
            .get_export_node_id(get_proc_export);

        let digest_hex = self.0
        .library()
        .mast_forest()
        .get_node_by_id(get_proc_mast_id)
        .ok_or_else(|| {
            JsValue::from_str(&format!(
                "Mast node for procedure '{}' not found",
                procedure_name
            ))
        })?
        .digest()
        .to_hex();

    // 4. Return the hex string.
        Ok(digest_hex)
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
