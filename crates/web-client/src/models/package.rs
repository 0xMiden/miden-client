use miden_client::vm::Package as NativePackage;
#[cfg(feature = "napi")]
use miden_client::{Deserializable, Serializable};
#[cfg(feature = "napi")]
use napi::bindgen_prelude::*;
#[cfg(feature = "wasm")]
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::library::Library;
use crate::models::program::Program;
use crate::platform;
use crate::prelude::*;
#[cfg(feature = "wasm")]
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

/// Compiled VM package containing libraries and metadata.
#[derive(Clone)]
#[bindings]
pub struct Package(NativePackage);

// Shared methods
#[bindings]
impl Package {
    /// Returns the underlying library of a `Package`.
    /// Fails if the package is not a library.
    #[bindings(wasm)]
    pub fn as_library(&self) -> platform::JsResult<Library> {
        if !self.0.is_library() {
            return Err(platform::error_from_string("Package does not contain a library"));
        }

        let native_library = self.0.unwrap_library();
        Ok((*native_library).clone().into())
    }

    /// Returns the underlying program of a `Package`.
    /// Fails if the package is not a program.
    #[bindings(wasm)]
    pub fn as_program(&self) -> platform::JsResult<Program> {
        if !self.0.is_program() {
            return Err(platform::error_from_string("Package does not contain a program"));
        }

        let native_program = self.0.unwrap_program();
        Ok((*native_program).clone().into())
    }
}

// Wasm-specific serialization
#[cfg(feature = "wasm")]
impl Package {
    /// Serializes the package into bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes a package from bytes.
    pub fn deserialize(bytes: &Uint8Array) -> platform::JsResult<Package> {
        deserialize_from_uint8array::<NativePackage>(bytes).map(Package)
    }
}

// Napi-specific serialization
#[cfg(feature = "napi")]
impl Package {
    /// Serializes the package into bytes.
    #[bindings(napi)]
    pub fn serialize(&self) -> Buffer {
        let bytes = self.0.to_bytes();
        Buffer::from(bytes)
    }

    /// Deserializes a package from bytes.
    #[bindings(napi(factory))]
    pub fn deserialize(bytes: Buffer) -> platform::JsResult<Package> {
        NativePackage::read_from_bytes(&bytes)
            .map(Package)
            .map_err(|e| {
                platform::error_with_context(e, "Error deserializing Package")
            })
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativePackage> for Package {
    fn from(native_package: NativePackage) -> Self {
        Package(native_package)
    }
}

impl From<&NativePackage> for Package {
    fn from(native_package: &NativePackage) -> Self {
        Package(native_package.clone())
    }
}

impl From<Package> for NativePackage {
    fn from(package: Package) -> Self {
        package.0
    }
}

impl From<&Package> for NativePackage {
    fn from(package: &Package) -> Self {
        package.0.clone()
    }
}
