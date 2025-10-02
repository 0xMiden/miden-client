use miden_objects::{
    AccountError,
    account::{
        AccountComponent as NativeAccountComponent, AccountComponentMetadata,
        AccountComponentTemplate as NativeAccountComponentTemplate,
        InitStorageData as NativeInitStorageData,
    },
    vm::{Package, SectionId},
};
use wasm_bindgen::prelude::*;

use crate::{js_error_with_context, models::account_component::AccountComponent};

/// WebAssembly wrapper for miden Package
///
/// This module provides functionality for working with Miden packages (.masp files),
/// enabling conversion from Package to AccountComponent for deploying custom accounts.
#[wasm_bindgen(js_name = "Package")]
pub struct WebPackage {
    inner: Package,
}

#[wasm_bindgen(js_class = "Package")]
impl WebPackage {
    /// Creates a Package from bytes (.masp file contents)
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: &[u8]) -> Result<WebPackage, JsValue> {
        let package = Package::read_from_bytes(bytes)
            .map_err(|e| js_error_with_context(e, "Failed to deserialize package from bytes"))?;

        Ok(WebPackage { inner: package })
    }

    /// Convert Package to AccountComponentTemplate
    /// This is the first step in the conversion chain for deploying custom accounts
    #[wasm_bindgen(js_name = "toAccountComponentTemplate")]
    pub fn to_account_component_template(&self) -> Result<AccountComponentTemplate, JsValue> {
        // Clone the package to convert it
        let package = self.inner.clone();

        // Convert Package to AccountComponentTemplate using TryFrom
        let template: NativeAccountComponentTemplate =
            package.try_into().map_err(|e: AccountError| {
                js_error_with_context(e, "Failed to convert package to account component template")
            })?;

        Ok(AccountComponentTemplate { inner: template })
    }

    /// Get the package name if available
    #[wasm_bindgen(js_name = "getName")]
    pub fn get_name(&self) -> Option<String> {
        self.inner.name().map(|n| n.to_string())
    }

    /// Get the package version
    #[wasm_bindgen(js_name = "getVersion")]
    pub fn get_version(&self) -> String {
        format!(
            "{}.{}.{}",
            self.inner.version().major,
            self.inner.version().minor,
            self.inner.version().patch
        )
    }

    /// Check if the package contains account component metadata
    #[wasm_bindgen(js_name = "hasAccountComponentMetadata")]
    pub fn has_account_component_metadata(&self) -> bool {
        self.inner
            .sections()
            .iter()
            .any(|section| section.id == SectionId::ACCOUNT_COMPONENT_METADATA)
    }
}

/// WebAssembly wrapper for AccountComponentTemplate
#[wasm_bindgen(js_name = "AccountComponentTemplate")]
pub struct AccountComponentTemplate {
    inner: NativeAccountComponentTemplate,
}

#[wasm_bindgen(js_class = "AccountComponentTemplate")]
impl AccountComponentTemplate {
    /// Create an AccountComponent from this template
    ///
    /// The template's component metadata might contain placeholders, which can be replaced by
    /// providing storage initialization data.
    #[wasm_bindgen(js_name = "toAccountComponent")]
    pub fn to_account_component(
        &self,
        init_storage_data: Option<InitStorageData>,
    ) -> Result<AccountComponent, JsValue> {
        let storage_data = init_storage_data
            .map(|d| d.inner)
            .unwrap_or_else(NativeInitStorageData::default);

        let component =
            NativeAccountComponent::from_template(&self.inner, &storage_data).map_err(|e| {
                js_error_with_context(e, "Failed to instantiate account component from template")
            })?;

        Ok(AccountComponent::from(component))
    }

    /// Get the supported account types for this template
    #[wasm_bindgen(js_name = "getSupportedTypes")]
    pub fn get_supported_types(&self) -> Vec<String> {
        self.inner
            .metadata()
            .supported_types()
            .iter()
            .map(|t| format!("{:?}", t))
            .collect()
    }

    /// Get the number of storage entries in the template
    #[wasm_bindgen(js_name = "getStorageEntriesCount")]
    pub fn get_storage_entries_count(&self) -> usize {
        self.inner.metadata().storage_entries().len()
    }

    /// Check if the template has any storage placeholders that need initialization
    #[wasm_bindgen(js_name = "hasStoragePlaceholders")]
    pub fn has_storage_placeholders(&self) -> bool {
        self.inner.metadata().storage_entries().iter().any(|entry| {
            // Check if the entry has placeholders (this is a simplified check)
            // In reality, you'd need to inspect the StorageEntry structure
            true // For now, assume all entries might have placeholders
        })
    }
}

/// WebAssembly wrapper for InitStorageData
///
/// This type is used to provide initial values for storage slots when instantiating
/// an AccountComponent from a template.
#[wasm_bindgen(js_name = "InitStorageData")]
pub struct InitStorageData {
    inner: NativeInitStorageData,
}

#[wasm_bindgen(js_class = "InitStorageData")]
impl InitStorageData {
    /// Create a new empty InitStorageData
    #[wasm_bindgen(constructor)]
    pub fn new() -> InitStorageData {
        InitStorageData { inner: NativeInitStorageData::default() }
    }

    /// Add a storage value by name
    ///
    /// @param name - The name of the storage value placeholder
    /// @param value - The value as a hex string (for Felt values) or array of hex strings (for Word values)
    #[wasm_bindgen(js_name = "addValue")]
    pub fn add_value(&mut self, name: &str, value: JsValue) -> Result<(), JsValue> {
        use miden_objects::Felt;
        use miden_objects::account::{FeltRepresentation, StorageValueName, WordRepresentation};

        let storage_name = StorageValueName::try_from(name)
            .map_err(|e| js_error_with_context(e, "Invalid storage value name"))?;

        // Try to parse as a single Felt value first
        if let Some(hex_str) = value.as_string() {
            let felt_value = Felt::try_from(hex_str.as_str()).map_err(|e| {
                js_error_with_context(e, "Failed to parse Felt value from hex string")
            })?;

            self.inner.insert(storage_name, FeltRepresentation(felt_value));
            return Ok(());
        }

        // Try to parse as an array of Felt values (Word)
        if let Ok(array) = js_sys::Array::try_from(value) {
            if array.length() != 4 {
                return Err(JsValue::from_str(
                    "Word values must be an array of exactly 4 hex strings",
                ));
            }

            let mut word_values = Vec::with_capacity(4);
            for i in 0..4 {
                let hex_str = array
                    .get(i)
                    .as_string()
                    .ok_or_else(|| JsValue::from_str("Word array must contain hex strings"))?;

                let felt = Felt::try_from(hex_str.as_str()).map_err(|e| {
                    js_error_with_context(e, &format!("Failed to parse Felt at index {}", i))
                })?;
                word_values.push(felt);
            }

            let word: [Felt; 4] = word_values
                .try_into()
                .map_err(|_| JsValue::from_str("Failed to convert to Word array"))?;

            self.inner.insert(storage_name, WordRepresentation(word));
            return Ok(());
        }

        Err(JsValue::from_str(
            "Value must be either a hex string (Felt) or an array of 4 hex strings (Word)",
        ))
    }

    /// Create InitStorageData from a JavaScript object
    ///
    /// @param data - An object where keys are storage value names and values are hex strings or arrays
    #[wasm_bindgen(js_name = "fromObject")]
    pub fn from_object(data: JsValue) -> Result<InitStorageData, JsValue> {
        let mut init_data = InitStorageData::new();

        let obj = js_sys::Object::try_from(data)
            .map_err(|_| JsValue::from_str("Input must be an object"))?;

        let entries = js_sys::Object::entries(&obj);
        for i in 0..entries.length() {
            let entry = js_sys::Array::from(&entries.get(i));
            let key = entry
                .get(0)
                .as_string()
                .ok_or_else(|| JsValue::from_str("Object keys must be strings"))?;
            let value = entry.get(1);

            init_data.add_value(&key, value)?;
        }

        Ok(init_data)
    }
}

/// Helper to validate package bytes
#[wasm_bindgen(js_name = "isValidPackageBytes")]
pub fn is_valid_package_bytes(bytes: &[u8]) -> bool {
    Package::read_from_bytes(bytes).is_ok()
}

/// Helper to extract account component metadata from package bytes
/// Returns null if the package doesn't contain account component metadata
#[wasm_bindgen(js_name = "extractAccountComponentMetadata")]
pub fn extract_account_component_metadata(bytes: &[u8]) -> Result<Option<String>, JsValue> {
    let package = Package::read_from_bytes(bytes)
        .map_err(|e| js_error_with_context(e, "Failed to read package"))?;

    let metadata = package.sections().iter().find_map(|section| {
        if section.id == SectionId::ACCOUNT_COMPONENT_METADATA {
            AccountComponentMetadata::read_from_bytes(&section.data).ok()
        } else {
            None
        }
    });

    Ok(metadata.map(|m| format!("{:?}", m)))
}

// CONVERSIONS
// ================================================================================================

impl From<Package> for WebPackage {
    fn from(package: Package) -> Self {
        WebPackage { inner: package }
    }
}

impl From<NativeAccountComponentTemplate> for AccountComponentTemplate {
    fn from(template: NativeAccountComponentTemplate) -> Self {
        AccountComponentTemplate { inner: template }
    }
}
