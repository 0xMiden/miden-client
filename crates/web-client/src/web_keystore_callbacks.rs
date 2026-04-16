use miden_client::auth::{
    AuthSecretKey,
    PublicKeyCommitment,
    Signature as NativeSignature,
    SigningInputs as NativeSigningInputs,
};
use miden_client::keystore::KeyStoreError;
use miden_client::utils::Deserializable;
use miden_client::{AuthenticationError, Word as NativeWord};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_futures::js_sys::{Function, Promise, Uint8Array};

use crate::models::auth_secret_key::AuthSecretKey as WebAuthSecretKey;
use crate::models::signature::Signature;
use crate::models::signing_inputs::SigningInputs;

/// Wrapper for the JavaScript `getKey` callback.
/// Expected JS signature: `(pubKeyCommitment: Uint8Array) => Promise<Uint8Array | null | undefined>
/// | Uint8Array | null | undefined`.
pub(crate) struct GetKeyCallback(pub(crate) Function);

impl GetKeyCallback {
    pub(crate) async fn get_secret_key(
        &self,
        pub_key_commitment: PublicKeyCommitment,
    ) -> Result<Option<AuthSecretKey>, KeyStoreError> {
        let pub_key_array = NativeWord::from(pub_key_commitment).as_bytes().to_vec();
        let call_result = self
            .0
            .call1(&JsValue::NULL, &JsValue::from(pub_key_array))
            .map_err(|err| KeyStoreError::StorageError(format!("JS getKey threw: {err:?}")))?;

        let resolved = if let Some(promise) = call_result.dyn_ref::<Promise>() {
            JsFuture::from(promise.clone()).await.map_err(|_| {
                KeyStoreError::StorageError("Failed to get secret key via callback".to_string())
            })?
        } else {
            call_result
        };

        if resolved.is_null() || resolved.is_undefined() {
            return Ok(None);
        }

        let u8_array = resolved.dyn_ref::<Uint8Array>().ok_or_else(|| {
            KeyStoreError::DecodingError("Expected secret key Uint8Array".to_string())
        })?;

        let secret_key_bytes = u8_array.to_vec();
        let secret_key = decode_secret_key_from_bytes(&secret_key_bytes)?;

        Ok(Some(secret_key))
    }
}

/// Wrapper for the JavaScript `insertKey` callback.
/// Expected JS signature: `(pubKeyCommitment: Uint8Array, secretKey: Uint8Array) => Promise<void> |
/// void`.
pub(crate) struct InsertKeyCallback(pub(crate) Function);

impl InsertKeyCallback {
    pub(crate) async fn insert_key(
        &self,
        secret_key: &WebAuthSecretKey,
    ) -> Result<(), KeyStoreError> {
        let pub_key_commitment: NativeWord = secret_key.public_key().to_commitment().into();
        let result = self
            .0
            .call2(
                &JsValue::NULL,
                &JsValue::from(pub_key_commitment.as_bytes().to_vec()),
                &JsValue::from(secret_key.serialize()),
            )
            .map_err(|err| KeyStoreError::StorageError(format!("JS insertKey threw: {err:?}")))?;

        if let Some(promise) = result.dyn_ref::<Promise>() {
            JsFuture::from(promise.clone()).await.map_err(|_| {
                KeyStoreError::StorageError("Failed to insert key via callback".to_string())
            })?;
        }
        Ok(())
    }
}

/// Wrapper for the JavaScript `sign` callback.
///
/// Expected JS signature: `(pubKeyCommitment: Uint8Array, commitment: Uint8Array) =>
/// Promise<number[] | string[]> | number[] | string[]`.
///
/// # Typed error convention
///
/// When the callback throws, consumers can attach a `reason` (string) and
/// optional `code` (string) property to the thrown `Error`. The raw thrown
/// value is captured and surfaced via [`crate::WebClient::last_auth_error`]
/// so a caller can distinguish (e.g.) "wallet locked" from "user rejected"
/// from "keystore unavailable" and retry or surface accordingly.
///
/// ```js
/// signCallback: async (pubKey, signingInputs) => {
///   if (vault.isLocked()) {
///     throw Object.assign(new Error("wallet locked"), { reason: "locked" });
///   }
///   return await sign(pubKey, signingInputs);
/// }
/// ```
pub(crate) struct SignCallback(pub(crate) Function);

/// Error returned by [`SignCallback::sign`]. Carries both the typed
/// [`AuthenticationError`] expected by miden-tx and the raw [`JsValue`]
/// thrown by the JS callback (when any), so callers can record it for
/// later inspection via [`WebClient::last_auth_error`].
pub(crate) struct SignCallbackError {
    pub(crate) auth_err: AuthenticationError,
    /// Raw JsValue thrown by the callback, or [`JsValue::NULL`] if the
    /// failure didn't originate from a JS throw (e.g. result type was
    /// wrong).
    pub(crate) raw: JsValue,
}

impl SignCallbackError {
    fn from_js(raw: JsValue, context: &str) -> Self {
        Self {
            auth_err: AuthenticationError::other(format!("{context}: {raw:?}")),
            raw,
        }
    }

    fn from_str(msg: &'static str) -> Self {
        Self {
            auth_err: AuthenticationError::other(msg),
            raw: JsValue::NULL,
        }
    }

    fn from_msg(msg: String) -> Self {
        Self {
            auth_err: AuthenticationError::other(msg),
            raw: JsValue::NULL,
        }
    }
}

impl SignCallback {
    pub(crate) async fn sign(
        &self,
        pub_key_commitment: NativeWord,
        signing_inputs: &NativeSigningInputs,
    ) -> Result<NativeSignature, SignCallbackError> {
        let signing_inputs_array = SigningInputs::from(signing_inputs).serialize();
        let pub_key_commitment_array = pub_key_commitment.as_bytes().to_vec();

        let call_result = self
            .0
            .call2(
                &JsValue::NULL,
                &JsValue::from(pub_key_commitment_array),
                &JsValue::from(signing_inputs_array),
            )
            .map_err(|err| SignCallbackError::from_js(err, "JS sign threw"))?;

        let resolved = if let Some(promise) = call_result.dyn_ref::<Promise>() {
            JsFuture::from(promise.clone())
                .await
                .map_err(|err| SignCallbackError::from_js(err, "sign callback promise rejected"))?
        } else {
            call_result
        };

        let bytes = resolved
            .dyn_ref::<Uint8Array>()
            .ok_or_else(|| SignCallbackError::from_str("sign callback must return a Uint8Array"))?;

        let signature = Signature::deserialize(bytes).map_err(|err| {
            SignCallbackError::from_msg(format!("Failed to decode callback signature: {err:?}"))
        })?;
        Ok(signature.into())
    }
}

pub(crate) fn decode_secret_key_from_bytes(bytes: &[u8]) -> Result<AuthSecretKey, KeyStoreError> {
    AuthSecretKey::read_from_bytes(bytes)
        .map_err(|err| KeyStoreError::DecodingError(format!("error reading secret key: {err:?}")))
}
