use js_export_macro::js_export;
use miden_client::vm::Package as NativePackage;

use crate::models::library::Library;
use crate::models::program::Program;
use crate::platform::{JsBytes, JsErr, from_str_err};
use crate::utils::{deserialize_from_bytes, serialize_to_bytes};

/// Compiled VM package containing libraries and metadata.
#[derive(Clone)]
#[js_export]
pub struct Package(NativePackage);

#[js_export]
impl Package {
    /// Serializes the package into bytes.
    pub fn serialize(&self) -> JsBytes {
        serialize_to_bytes(&self.0)
    }

    /// Deserializes a package from bytes.
    pub fn deserialize(bytes: JsBytes) -> Result<Package, JsErr> {
        deserialize_from_bytes::<NativePackage>(&bytes).map(Package)
    }

    /// Returns the underlying library of a `Package`.
    /// Fails if the package is not a library.
    #[js_export(js_name = "asLibrary")]
    pub fn as_library(&self) -> Result<Library, JsErr> {
        if !self.0.is_library() {
            return Err(from_str_err("Package does not contain a library"));
        }

        let native_library = self.0.unwrap_library();
        Ok((*native_library).clone().into())
    }

    /// Returns the underlying program of a `Package`.
    /// Fails if the package is not a program.
    #[js_export(js_name = "asProgram")]
    pub fn as_program(&self) -> Result<Program, JsErr> {
        if !self.0.is_program() {
            return Err(from_str_err("Package does not contain a program"));
        }

        let native_program = self.0.unwrap_program();
        Ok((*native_program).clone().into())
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
