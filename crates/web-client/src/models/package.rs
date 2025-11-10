use miden_client::vm::Package as NativePackage;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

#[derive(Clone)]
#[wasm_bindgen]
pub struct Package(NativePackage);

#[wasm_bindgen]
impl Package {
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    pub fn deserialize(bytes: &Uint8Array) -> Result<Package, JsValue> {
        deserialize_from_uint8array::<NativePackage>(bytes).map(Package)
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
