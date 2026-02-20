//! Unified prelude for wasm-bindgen and napi-derive attributes.
//!
//! Import this instead of conditionally importing wasm_bindgen::prelude or napi_derive.

// Re-export alloc types for no_std compatibility (also works with std)
pub use alloc::boxed::Box;
pub use alloc::format;
pub use alloc::string::{String, ToString};
pub use alloc::sync::Arc;
pub use alloc::vec;
pub use alloc::vec::Vec;
pub use alloc::borrow::ToOwned;

// Re-export the unified bindings macro
pub use miden_bindings_macro::bindings;

// Re-export wasm-bindgen for wasm feature (still needed for some cases)
#[cfg(feature = "wasm")]
pub use wasm_bindgen::prelude::*;

// Re-export napi-derive for napi feature (still needed for some cases)
#[cfg(feature = "napi")]
pub use napi_derive::napi;

// Common re-exports that both need
pub(crate) use crate::platform;
pub use crate::platform::{JsResult, JsBytes};
