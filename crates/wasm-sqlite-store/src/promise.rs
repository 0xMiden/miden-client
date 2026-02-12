use serde::de::DeserializeOwned;
use serde_wasm_bindgen::from_value;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_futures::js_sys::Promise;

use crate::StoreError;

/// Awaits a JavaScript `Promise` and returns the raw `JsValue` on success.
#[allow(dead_code)]
pub(crate) async fn await_js_value(promise: Promise, ctx: &str) -> Result<JsValue, StoreError> {
    JsFuture::from(promise)
        .await
        .map_err(|js_error| StoreError::DatabaseError(format!("{ctx}: {js_error:?}")))
}

/// Awaits a JavaScript `Promise` and deserializes the result into `T`.
#[allow(dead_code)]
pub(crate) async fn await_js<T>(promise: Promise, ctx: &str) -> Result<T, StoreError>
where
    T: DeserializeOwned,
{
    let js_value = await_js_value(promise, ctx).await?;
    from_value(js_value)
        .map_err(|err| StoreError::DatabaseError(format!("failed to deserialize ({ctx}): {err:?}")))
}

/// Awaits a JavaScript `Promise` and discards the result.
#[allow(dead_code)]
pub(crate) async fn await_ok(promise: Promise, ctx: &str) -> Result<(), StoreError> {
    let _ = await_js_value(promise, ctx).await?;
    Ok(())
}
