use miden_client::auth::{
    AuthSecretKey,
    PublicKey as NativePublicKey,
    Signature as NativeSignature,
    SigningInputs as NativeSigningInputs,
};
use miden_client::keystore::KeyStoreError;
use miden_client::utils::Deserializable;
use miden_client::{AuthenticationError, Word as NativeWord};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_futures::js_sys::{Function, Promise, Uint8Array};

use crate::models::secret_key::SecretKey;
use crate::models::signature::Signature;
use crate::models::signing_inputs::SigningInputs;

/// Wrapper for the JavaScript `getKey` callback.
/// Expected JS signature: `(pubKeyCommitment: Uint8Array) => Promise<Uint8Array | null | undefined>
/// | Uint8Array | null | undefined`.
pub(crate) struct GetKeyCallback(pub(crate) Function);

impl GetKeyCallback {
    pub(crate) async fn get_secret_key(
        &self,
        pub_key_commitment: NativeWord,
    ) -> Result<Option<AuthSecretKey>, KeyStoreError> {
        let pub_key_array = pub_key_commitment.as_bytes().to_vec();
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
    pub(crate) async fn insert_key(&self, secret_key: &SecretKey) -> Result<(), KeyStoreError> {
        let pub_key: NativePublicKey = secret_key.public_key().into();
        let pub_key_commitment: NativeWord = pub_key.to_commitment().into();
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
/// Expected JS signature: `(pubKeyCommitment: Uint8Array, commitment: Uint8Array) =>
/// Promise<number[] | string[]> | number[] | string[]`.
pub(crate) struct SignCallback(pub(crate) Function);

impl SignCallback {
    pub(crate) async fn sign(
        &self,
        pub_key_commitment: NativeWord,
        signing_inputs: &NativeSigningInputs,
    ) -> Result<NativeSignature, AuthenticationError> {
        let signing_inputs_array = SigningInputs::from(signing_inputs).serialize();
        let pub_key_commitment_array = pub_key_commitment.as_bytes().to_vec();

        let call_result = self
            .0
            .call2(
                &JsValue::NULL,
                &JsValue::from(pub_key_commitment_array),
                &JsValue::from(signing_inputs_array),
            )
            .map_err(|err| AuthenticationError::other(format!("JS sign threw: {err:?}")))?;

        let resolved = if let Some(promise) = call_result.dyn_ref::<Promise>() {
            JsFuture::from(promise.clone()).await.map_err(|err| {
                AuthenticationError::other(format!("Failed to sign via callback: {err:?}"))
            })?
        } else {
            call_result
        };

        let bytes = resolved
            .dyn_ref::<Uint8Array>()
            .ok_or_else(|| AuthenticationError::other("sign callback must return a Uint8Array"))?;

        let signature = Signature::deserialize(bytes).map_err(|err| {
            AuthenticationError::other(format!("Failed to sign via callback: {err:?}"))
        })?;
        Ok(signature.into())
    }
}

pub(crate) fn decode_secret_key_from_bytes(bytes: &[u8]) -> Result<AuthSecretKey, KeyStoreError> {
    AuthSecretKey::read_from_bytes(bytes)
        .map_err(|err| KeyStoreError::DecodingError(format!("error reading secret key: {err:?}")))
}
