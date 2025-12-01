use miden_client::vm::PackageExport as NativePackageExport;
use wasm_bindgen::prelude::*;

use crate::models::word::Word;

#[derive(Clone)]
#[wasm_bindgen]
pub struct FunctionType {
    abi: String,
    params: Vec<String>,
    results: Vec<String>,
}

#[wasm_bindgen]
impl FunctionType {
    #[wasm_bindgen(constructor)]
    pub fn new(abi: String, params: Vec<String>, results: Vec<String>) -> FunctionType {
        FunctionType { abi, params, results }
    }

    #[wasm_bindgen(getter)]
    pub fn abi(&self) -> String {
        self.abi.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn params(&self) -> Vec<String> {
        self.params.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn results(&self) -> Vec<String> {
        self.results.clone()
    }
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct PackageExport(NativePackageExport);

#[wasm_bindgen]
impl PackageExport {
    pub fn name(&self) -> String {
        self.0.name.name.as_str().to_string()
    }

    pub fn digest(&self) -> Word {
        self.0.digest.into()
    }

    pub fn signature(&self) -> FunctionType {
        let native_function_type = self.0.signature.clone().unwrap();
        let abi = native_function_type.abi.to_string();
        let params = native_function_type.params.iter().map(|ty| ty.to_string()).collect();
        let results = native_function_type.results.iter().map(|ty| ty.to_string()).collect();
        FunctionType::new(abi, params, results)
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativePackageExport> for PackageExport {
    fn from(native_package_export: NativePackageExport) -> Self {
        PackageExport(native_package_export)
    }
}

impl From<&NativePackageExport> for PackageExport {
    fn from(native_package_export: &NativePackageExport) -> Self {
        PackageExport(native_package_export.clone())
    }
}
