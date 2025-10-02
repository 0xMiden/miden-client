use wasm_bindgen::prelude::*;
use miden_client::account::component::{AccountComponentTemplate, AccountComponent, Package};
use miden_client::{Deserializable, Serializable};

use crate::models::account_component::AccountComponent as WebAccountComponent;

/// WebClient wrapper for miden Package type
#[wasm_bindgen(js_name = "Package")]
pub struct WebPackage {
    inner: Package,
}

#[wasm_bindgen(js_class = "Package")]
impl WebPackage {
    /// Creates a Package from a Uint8Array, deserializing the underlying Rust struct from an array of bytes.
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: &[u8]) -> Result<WebPackage, JsValue> {
        let package = Package::read_from_bytes(bytes)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize package: {}", e)))?;
        
        Ok(WebPackage { inner: package })
    }

    /// Get the name of the package
    #[wasm_bindgen(js_name = "getName")]
    pub fn get_name(&self) -> String {
        self.inner.name().to_string()
    }

    /// Get the version of the package
    #[wasm_bindgen(js_name = "getVersion")]
    pub fn get_version(&self) -> String {
        format!("{}.{}.{}", 
            self.inner.version().major,
            self.inner.version().minor,
            self.inner.version().patch
        )
    }

    /// Query the package to get access to exported procedures names
    /// Returns an array of procedure names (e.g., ["increment_count", "decrement_count"])
    #[wasm_bindgen(js_name = "getExportedProcedureNames")]
    pub fn get_exported_procedure_names(&self) -> Vec<String> {
        self.inner
            .mast()
            .procedures()
            .map(|proc_info| proc_info.name.to_string())
            .collect()
    }

    /// Query the package to get access to exported procedures with their signatures
    /// Returns a JavaScript object with procedure names as keys and signatures as values
    #[wasm_bindgen(js_name = "getExportedProcedures")]
    pub fn get_exported_procedures(&self) -> Result<JsValue, JsValue> {
        let procedures = js_sys::Object::new();
        
        for proc_info in self.inner.mast().procedures() {
            let name = proc_info.name.to_string();
            let signature = format!("func({}) -> {}", 
                proc_info.num_locals,
                if proc_info.num_outputs > 0 {
                    format!("felt{}", if proc_info.num_outputs > 1 { 
                        format!("[{}]", proc_info.num_outputs) 
                    } else { 
                        String::new() 
                    })
                } else {
                    "()".to_string()
                }
            );
            
            js_sys::Reflect::set(
                &procedures,
                &JsValue::from_str(&name),
                &JsValue::from_str(&signature),
            )?;
        }
        
        Ok(procedures.into())
    }

    /// Check if the package contains account component metadata
    #[wasm_bindgen(js_name = "hasAccountComponentMetadata")]
    pub fn has_account_component_metadata(&self) -> bool {
        // Check if the package can be converted to an AccountComponentTemplate
        // This is the best way to check for account component metadata
        AccountComponentTemplate::try_from(self.inner.clone()).is_ok()
    }

    /// Convert the package to an AccountComponentTemplate
    /// This will fail if the package doesn't contain account component metadata
    #[wasm_bindgen(js_name = "toAccountComponentTemplate")]
    pub fn to_account_component_template(&self) -> Result<AccountComponentTemplate, JsValue> {
        AccountComponentTemplate::try_from(self.inner.clone())
            .map_err(|e| JsValue::from_str(&format!("Failed to convert package to AccountComponentTemplate: {}", e)))
    }

    /// Create an AccountComponent from the package
    /// This enables deploying custom accounts developed in Rust
    #[wasm_bindgen(js_name = "toAccountComponent")]
    pub fn to_account_component(&self) -> Result<WebAccountComponent, JsValue> {
        // First convert to AccountComponentTemplate
        let template = self.to_account_component_template()?;
        
        // Then create AccountComponent from the template
        // Note: This creates a component with empty storage slots
        // In a real implementation, you might want to pass storage data
        let component = AccountComponent::from_template(&template, &Default::default())
            .map_err(|e| JsValue::from_str(&format!("Failed to create AccountComponent from package: {}", e)))?;
        
        Ok(WebAccountComponent::from(component))
    }

    /// Serialize the package back to bytes
    #[wasm_bindgen(js_name = "toBytes")]
    pub fn to_bytes(&self) -> Result<Vec<u8>, JsValue> {
        self.inner.to_bytes()
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize package: {}", e)))
    }
}

// Helper function to expose Package type checking
#[wasm_bindgen(js_name = "isValidPackageBytes")]
pub fn is_valid_package_bytes(bytes: &[u8]) -> bool {
    Package::read_from_bytes(bytes).is_ok()
}