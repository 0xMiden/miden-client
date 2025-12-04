use miden_client::vm::PackageManifest as NativePackageManifest;
use wasm_bindgen::prelude::*;

use crate::models::package_export::PackageExport;

#[derive(Clone)]
#[wasm_bindgen]
pub struct PackageManifest(NativePackageManifest);

#[wasm_bindgen]
impl PackageManifest {
    pub fn exports(&self) -> Vec<PackageExport> {
        self.0.exports().map(Into::into).collect()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativePackageManifest> for PackageManifest {
    fn from(native_package_manifest: NativePackageManifest) -> Self {
        PackageManifest(native_package_manifest)
    }
}

impl From<&NativePackageManifest> for PackageManifest {
    fn from(native_package_manifest: &NativePackageManifest) -> Self {
        PackageManifest(native_package_manifest.clone())
    }
}
