use miden_client::account::component::{
    AccountComponent as NativeAccountComponent,
    AccountComponentMetadata,
    AccountComponentTemplate,
    InitStorageData,
};
use miden_client::vm::Package as NativePackage;
use wasm_bindgen::prelude::*;

use crate::js_error_with_context;
use crate::models::account_component::AccountComponent;

/// WebAssembly wrapper for miden Package
///
/// This module provides functionality for working with Miden packages (.masp files),
/// enabling conversion from Package to `AccountComponent` for deploying custom accounts.
#[wasm_bindgen]
pub struct Package {
    inner: NativePackage,
}

#[wasm_bindgen]
impl Package {
    /// Creates a Package from bytes (.masp file contents)
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: &[u8]) -> Result<Package, JsValue> {
        use miden_client::utils::Deserializable;

        let package = NativePackage::read_from_bytes(bytes)
            .map_err(|e| js_error_with_context(e, "Failed to deserialize package from bytes"))?;

        Ok(Package { inner: package })
    }

    /// Get the package name if available
    #[wasm_bindgen(js_name = "getName")]
    pub fn get_name(&self) -> Option<String> {
        Some(self.inner.name.clone())
    }

    /// Check if the package contains account component metadata
    #[wasm_bindgen(js_name = "hasAccountComponentMetadata")]
    pub fn has_account_component_metadata(&self) -> bool {
        self.inner.account_component_metadata_bytes.is_some()
    }

    /// Convert the Package to an `AccountComponent` using default initialization data
    ///
    /// This method converts a Package containing account component metadata into an
    /// `AccountComponent` that can be used to build accounts.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The package does not contain account component metadata
    /// - The package cannot be converted to an `AccountComponentTemplate`
    /// - The component creation fails
    #[wasm_bindgen(js_name = "toAccountComponent")]
    pub fn to_account_component(&self) -> Result<AccountComponent, JsValue> {
        self.to_account_component_with_init_data(&JsValue::NULL)
    }

    /// Convert the Package to an `AccountComponent` with initialization data
    ///
    /// This method converts a Package containing account component metadata into an
    /// `AccountComponent` using the provided initialization data for storage slots.
    ///
    /// # Arguments
    ///
    /// * `init_data_js` - Optional JavaScript object containing initialization data for storage
    ///   slots. Pass null or undefined to use default initialization.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The package does not contain account component metadata
    /// - The package cannot be converted to an `AccountComponentTemplate`
    /// - The storage initialization fails due to invalid or missing data
    /// - The component creation fails
    #[wasm_bindgen(js_name = "toAccountComponentWithInitData")]
    pub fn to_account_component_with_init_data(
        &self,
        init_data_js: &JsValue,
    ) -> Result<AccountComponent, JsValue> {
        // Convert Package to AccountComponentTemplate
        let template = AccountComponentTemplate::try_from(self.inner.clone()).map_err(|e| {
            js_error_with_context(e, "Failed to convert package to account component template")
        })?;

        // Parse initialization data if provided
        let init_data = if init_data_js.is_null() || init_data_js.is_undefined() {
            InitStorageData::default()
        } else {
            // For now, we use default initialization data
            // In the future, we could parse the JavaScript object to build InitStorageData
            // This would involve converting JS object properties to storage slot values
            InitStorageData::default()
        };

        // Create AccountComponent from template
        let native_component = NativeAccountComponent::from_template(&template, &init_data)
            .map_err(|e| {
                js_error_with_context(e, "Failed to create account component from template")
            })?;

        Ok(AccountComponent::from(native_component))
    }

    /// Get a description of the package metadata
    ///
    /// Returns a JSON string containing information about the package and its
    /// account component metadata if present.
    #[wasm_bindgen(js_name = "getMetadataDescription")]
    pub fn get_metadata_description(&self) -> String {
        use miden_client::utils::Deserializable;

        let mut description = format!(r#"{{"name":"{}","version":"0.0.0""#, self.inner.name);

        // Try to extract and describe account component metadata
        if let Some(metadata_bytes) = &self.inner.account_component_metadata_bytes
            && let Ok(metadata) = AccountComponentMetadata::read_from_bytes(metadata_bytes)
        {
            use core::fmt::Write;
            let _ = write!(
                description,
                r#","component":{{"name":"{}","description":"{}","version":"{}","supported_types":{:?}}}"#,
                metadata.name(),
                metadata.description(),
                metadata.version(),
                metadata.supported_types().iter().map(|t| format!("{t:?}")).collect::<Vec<_>>()
            );
        }

        description.push('}');
        description
    }
}

/// Helper to validate package bytes
#[wasm_bindgen(js_name = "isValidPackageBytes")]
pub fn is_valid_package_bytes(bytes: &[u8]) -> bool {
    use miden_client::utils::Deserializable;
    NativePackage::read_from_bytes(bytes).is_ok()
}

/// Helper to extract account component metadata from package bytes
/// Returns a JSON string with metadata information, or null if the package doesn't contain account
/// component metadata
#[wasm_bindgen(js_name = "extractAccountComponentMetadata")]
pub fn extract_account_component_metadata(bytes: &[u8]) -> Result<Option<String>, JsValue> {
    use miden_client::utils::Deserializable;

    let package = NativePackage::read_from_bytes(bytes)
        .map_err(|e| js_error_with_context(e, "Failed to deserialize package from bytes"))?;

    // Check if package has account component metadata
    match &package.account_component_metadata_bytes {
        Some(metadata_bytes) => {
            let metadata =
                AccountComponentMetadata::read_from_bytes(metadata_bytes).map_err(|e| {
                    js_error_with_context(e, "Failed to deserialize account component metadata")
                })?;

            let json = format!(
                r#"{{"name":"{}","description":"{}","version":"{}","supported_types":{:?}}}"#,
                metadata.name(),
                metadata.description(),
                metadata.version(),
                metadata.supported_types().iter().map(|t| format!("{t:?}")).collect::<Vec<_>>()
            );

            Ok(Some(json))
        },
        None => Ok(None),
    }
}
