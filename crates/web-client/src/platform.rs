//! Platform abstraction layer for wasm/napi interop.
//!
//! Provides unified type aliases and helper functions so that most model and client
//! code can be written once, with `#[cfg_attr]` switching only the proc-macro attributes.

use core::error::Error;

// ================================================================================================
// UNIFIED TYPE ALIASES
// ================================================================================================

/// Unified error type for JavaScript-facing conversions.
#[cfg(feature = "wasm")]
pub type PlatformError = wasm_bindgen::JsValue;
#[cfg(feature = "napi")]
pub type PlatformError = napi::Error;

/// Unified result type for JavaScript-facing methods.
#[cfg(feature = "wasm")]
pub type JsResult<T> = Result<T, wasm_bindgen::JsValue>;
#[cfg(feature = "napi")]
pub type JsResult<T> = napi::Result<T>;

/// Unified byte buffer type for serialization/deserialization.
#[cfg(feature = "wasm")]
pub type JsBytes = wasm_bindgen_futures::js_sys::Uint8Array;
#[cfg(feature = "napi")]
pub type JsBytes = napi::bindgen_prelude::Buffer;

// ================================================================================================
// ERROR HELPERS
// ================================================================================================

/// Creates a JS-compatible error from any `Error` value, walking the error chain.
///
/// On wasm, this produces a `JsValue` wrapping a `JsError`.
/// On napi, this produces a `napi::Error`.
#[cfg(feature = "wasm")]
pub(crate) fn error_with_context<T>(err: T, context: &str) -> wasm_bindgen::JsValue
where
    T: Error + 'static,
{
    // Re-use the existing implementation in lib.rs during the transition.
    // After Phase 3, all callers will use this function directly.
    crate::js_error_with_context(err, context)
}

#[cfg(feature = "napi")]
pub(crate) fn error_with_context<T: Error>(err: T, context: &str) -> napi::Error {
    crate::napi_error_with_context(err, context)
}

/// Creates a JS-compatible error from a string message.
#[cfg(feature = "wasm")]
pub(crate) fn error_from_string(msg: &str) -> wasm_bindgen::JsValue {
    wasm_bindgen::JsValue::from_str(msg)
}

#[cfg(feature = "napi")]
pub(crate) fn error_from_string(msg: &str) -> napi::Error {
    napi::Error::from_reason(msg)
}

// ================================================================================================
// SERIALIZATION HELPERS
// ================================================================================================

use miden_client::utils::Serializable;

/// Serializes a value into a platform-appropriate byte buffer.
#[cfg(feature = "wasm")]
pub fn serialize_to_bytes<T: Serializable>(value: &T) -> JsBytes {
    let mut buffer = alloc::vec::Vec::new();
    value.write_into(&mut buffer);
    JsBytes::from(&buffer[..])
}

#[cfg(feature = "napi")]
pub fn serialize_to_bytes<T: Serializable>(value: &T) -> JsBytes {
    let bytes = value.to_bytes();
    JsBytes::from(bytes)
}

use miden_client::utils::Deserializable;

/// Deserializes a value from a platform-appropriate byte buffer.
#[cfg(feature = "wasm")]
pub fn deserialize_from_bytes<T: Deserializable>(bytes: &JsBytes) -> JsResult<T> {
    let vec = bytes.to_vec();
    let mut reader = miden_client::SliceReader::new(&vec);
    let context = alloc::format!("failed to deserialize {}", core::any::type_name::<T>());
    T::read_from(&mut reader).map_err(|e| error_with_context(e, &context))
}

#[cfg(feature = "napi")]
pub fn deserialize_from_bytes<T: Deserializable>(bytes: &JsBytes) -> JsResult<T> {
    let context = std::format!("failed to deserialize {}", std::any::type_name::<T>());
    T::read_from_bytes(bytes).map_err(|e| error_with_context(e, &context))
}
