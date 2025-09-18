use miden_lib::utils::ScriptBuilder as NativeScriptBuilder;
use wasm_bindgen::prelude::*;

#[derive(Clone)]
#[wasm_bindgen(inspectable)]
pub struct ScriptBuilder(NativeScriptBuilder);

#[wasm_bindgen]
impl ScriptBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new(in_debug_mode: bool) -> Self {
        Self(NativeScriptBuilder::new(in_debug_mode))
    }

    #[wasm_bindgen(js_name = "linkModule")]
    pub fn link_module(&mut self, module_path: &str, module_code: &str) -> Self {
        self.0.link_module(module_path, module_code)
    }

    #[wasm_bindgen(js_name = "linkModule")]
    pub fn link_module(&mut self, module_path: &str, module_code: &str) -> Self {
        self.0.link_module(module_path, module_code)
    }
}
