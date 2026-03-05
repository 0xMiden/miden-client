// Platform abstraction layer for browser (wasm_bindgen) vs Node.js (napi-rs).
//
// Provides type aliases and helper functions that abstract over the differences
// between the two binding technologies.

// ERROR TYPES
// ================================================================================================

/// Platform-specific JS error type.
#[cfg(feature = "browser")]
pub(crate) type JsErr = wasm_bindgen::JsValue;

/// Platform-specific JS error type.
#[cfg(feature = "nodejs")]
pub(crate) type JsErr = napi::Error;

/// Create an error from a string message.
#[cfg(feature = "browser")]
pub(crate) fn from_str_err(msg: &str) -> JsErr {
    wasm_bindgen::JsValue::from_str(msg)
}

/// Create an error from a string message.
#[cfg(feature = "nodejs")]
pub(crate) fn from_str_err(msg: &str) -> JsErr {
    napi::Error::from_reason(msg)
}

// BYTE TYPES
// ================================================================================================

/// Platform-specific byte array type for serialization/deserialization.
#[cfg(feature = "browser")]
pub(crate) type JsBytes = js_sys::Uint8Array;

/// Platform-specific byte array type for serialization/deserialization.
#[cfg(feature = "nodejs")]
pub(crate) type JsBytes = napi::bindgen_prelude::Buffer;

/// Convert a byte slice to the platform-specific byte array type.
#[cfg(feature = "browser")]
pub(crate) fn bytes_to_js(bytes: &[u8]) -> JsBytes {
    js_sys::Uint8Array::from(bytes)
}

/// Convert a byte slice to the platform-specific byte array type.
#[cfg(feature = "nodejs")]
pub(crate) fn bytes_to_js(bytes: Vec<u8>) -> JsBytes {
    napi::bindgen_prelude::Buffer::from(bytes)
}

/// Convert a platform-specific byte array to a Vec<u8>.
#[cfg(feature = "browser")]
pub(crate) fn js_to_bytes(js_bytes: &JsBytes) -> Vec<u8> {
    js_bytes.to_vec()
}

/// Convert a platform-specific byte array to a Vec<u8>.
#[cfg(feature = "nodejs")]
pub(crate) fn js_to_bytes(js_bytes: &JsBytes) -> Vec<u8> {
    js_bytes.to_vec()
}

// INTERIOR MUTABILITY
// ================================================================================================

/// Platform-specific async-compatible interior mutability wrapper.
///
/// - Browser (WASM): Uses `RefCell` (single-threaded, no contention).
/// - Node.js (native): Uses `tokio::sync::Mutex` (async-safe for napi's tokio runtime).
#[cfg(feature = "browser")]
pub(crate) struct AsyncCell<T>(std::cell::RefCell<T>);

#[cfg(feature = "nodejs")]
pub(crate) struct AsyncCell<T>(tokio::sync::Mutex<SendWrapper<T>>);

#[cfg(feature = "browser")]
impl<T> AsyncCell<T> {
    pub fn new(val: T) -> Self {
        Self(std::cell::RefCell::new(val))
    }

    pub async fn lock(&self) -> std::cell::RefMut<'_, T> {
        self.0.borrow_mut()
    }
}

#[cfg(feature = "nodejs")]
impl<T> AsyncCell<T> {
    pub fn new(val: T) -> Self {
        Self(tokio::sync::Mutex::new(SendWrapper(val)))
    }

    pub async fn lock(&self) -> impl core::ops::DerefMut<Target = T> + '_ {
        tokio::sync::MutexGuard::map(self.0.lock().await, |w| &mut w.0)
    }
}

/// Wrapper that unsafely implements `Send` and `Sync` for its inner value.
///
/// SAFETY: This is only used in the Node.js (napi-rs) build, where the JavaScript
/// runtime is single-threaded. The inner value is protected by a `tokio::sync::Mutex`
/// and will never be truly accessed from multiple threads concurrently. napi-rs requires
/// `Send` bounds on futures and return types, but the actual execution is sequential.
#[cfg(feature = "nodejs")]
pub(crate) struct SendWrapper<T>(pub T);

#[cfg(feature = "nodejs")]
unsafe impl<T> Send for SendWrapper<T> {}
#[cfg(feature = "nodejs")]
unsafe impl<T> Sync for SendWrapper<T> {}

// NUMERIC TYPES
// ================================================================================================

/// Platform-specific unsigned 64-bit integer type for JS interop.
///
/// - Browser (wasm_bindgen): `u64` maps to JavaScript `BigInt`.
/// - Node.js (napi-rs): `f64` maps to JavaScript `number` (safe for values up to 2^53).
#[cfg(feature = "browser")]
pub type JsU64 = u64;

#[cfg(feature = "nodejs")]
pub type JsU64 = f64;

// FUTURE SEND WRAPPER
// ================================================================================================

/// On browser (WASM), futures are not `Send` and don't need to be — just pass through.
#[cfg(feature = "browser")]
pub(crate) fn maybe_wrap_send<F: std::future::Future>(
    future: F,
) -> impl std::future::Future<Output = F::Output> {
    future
}

/// On Node.js, napi-rs requires `Send` futures for its tokio runtime.
/// This unsafely asserts `Send` — safe because napi's JS runtime is single-threaded,
/// so the non-Send trait objects never actually cross thread boundaries.
#[cfg(feature = "nodejs")]
pub(crate) fn maybe_wrap_send<F: std::future::Future>(
    future: F,
) -> impl std::future::Future<Output = F::Output> + Send {
    struct AssertSend<F>(F);
    unsafe impl<F> Send for AssertSend<F> {}
    impl<F: std::future::Future> std::future::Future for AssertSend<F> {
        type Output = F::Output;
        fn poll(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            unsafe { self.map_unchecked_mut(|s| &mut s.0) }.poll(cx)
        }
    }
    AssertSend(future)
}

// CLIENT AUTH TYPE
// ================================================================================================

/// Platform-specific client authenticator type.
#[cfg(feature = "browser")]
pub(crate) type ClientAuth = crate::web_keystore::WebKeyStore<miden_client::crypto::RpoRandomCoin>;

/// Platform-specific client authenticator type.
#[cfg(feature = "nodejs")]
pub(crate) type ClientAuth = miden_client::keystore::FilesystemKeyStore;
