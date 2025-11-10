use alloc::string::String;
use alloc::vec::Vec;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{js_sys, wasm_bindgen};

// ChainData IndexedDB Operations
include!(concat!(env!("OUT_DIR"), "/generated_js_bindings/chain_data_js_bindings.rs"));
