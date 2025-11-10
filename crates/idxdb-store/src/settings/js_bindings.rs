use alloc::string::String;
use alloc::vec::Vec;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys, wasm_bindgen};

include!(concat!(env!("OUT_DIR"), "/generated_js_bindings/settings_js_bindings.rs"));
