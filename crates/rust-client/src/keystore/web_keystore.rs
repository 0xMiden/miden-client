use alloc::{string::ToString, sync::Arc, vec::Vec};

use super::KeyStoreError;
use crate::keystore::common::hash_pub_key;
use crate::{
    AuthenticationError, Felt, Word,
    account::AccountDelta,
    auth::{AuthSecretKey, TransactionAuthenticator},
    crypto::Digest,
    utils::RwLock,
};
use js_sys::{ArrayBuffer, Uint8Array};
use miden_lib::utils::{Deserializable, Serializable};
use pollster::FutureExt as _;
use rand::Rng;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    DedicatedWorkerGlobalScope, FileSystemDirectoryHandle, FileSystemFileHandle,
    FileSystemGetDirectoryOptions, FileSystemGetFileOptions, FileSystemWritableFileStream,
};

/// A web-based keystore that stores keys in [Origin Private File System](https://developer.mozilla.org/en-US/docs/Web/API/File_System_API/Origin_private_file_system)
/// and provides transaction authentication functionality.
#[derive(Clone)]
pub struct WebKeyStore<R: Rng> {
    /// The random number generator used to generate signatures.
    rng: Arc<RwLock<R>>,
    /// OPFS directory handle
    keys_directory: FileSystemDirectoryHandle,
}

impl<R: Rng> WebKeyStore<R> {
    /// Creates a new instance of the web keystore with the provided RNG.
    pub async fn with_rng(rng: R) -> Result<Self, JsValue> {
        // TODO(Maks) extend with user provided path?
        let storage =
            if let Ok(worker_scope) = js_sys::global().dyn_into::<DedicatedWorkerGlobalScope>() {
                worker_scope.navigator().storage()
            } else if let Some(window) = web_sys::window() {
                window.navigator().storage()
            } else {
                return Err(JsValue::from_str("cannot get the browser storage"));
            };

        let root_directory =
            FileSystemDirectoryHandle::from(JsFuture::from(storage.get_directory()).await?);
        let fs_options = FileSystemGetDirectoryOptions::new();
        fs_options.set_create(true);
        let keys_directory = FileSystemDirectoryHandle::from(
            JsFuture::from(
                root_directory.get_directory_handle_with_options("keystore", &fs_options),
            )
            .await?,
        );
        Ok(WebKeyStore {
            rng: Arc::new(RwLock::new(rng)),
            keys_directory,
        })
    }

    pub async fn add_key(&self, key: &AuthSecretKey) -> Result<(), JsValue> {
        web_sys::console::log_1(&"== adding key".into());
        let pub_key = match &key {
            AuthSecretKey::RpoFalcon512(k) => Word::from(k.public_key()),
        };
        let filename = hash_pub_key(pub_key);
        let secret_key_hex = hex::encode(key.to_bytes());

        web_sys::console::log_1(&"== creating file".into());
        let fs_options = FileSystemGetFileOptions::new();
        fs_options.set_create(true);
        let file_handle = FileSystemFileHandle::from(
            JsFuture::from(
                self.keys_directory.get_file_handle_with_options(&filename, &fs_options),
            )
            .await?,
        );

        web_sys::console::log_1(&"== creaing file stream".into());
        let file_stream = FileSystemWritableFileStream::from(
            JsFuture::from(file_handle.create_writable()).await?,
        );
        let slice = secret_key_hex.as_bytes();
        let expected_num_bytes = slice.len();
        web_sys::console::log_1(&"== writing to file".into());
        let num_bytes: f64 = JsFuture::from(file_stream.write_with_u8_array(slice)?)
            .await?
            .as_f64()
            .ok_or_else(|| JsValue::from_str("cannot save key"))?;

        JsFuture::from(file_stream.close()).await?;
        if expected_num_bytes != num_bytes as usize {
            JsFuture::from(self.keys_directory.remove_entry(&filename)).await?;
            Err(JsValue::from_str("cannot save key - corrupted"))
        } else {
            Ok(())
        }
    }

    pub async fn get_key(&self, pub_key: Word) -> Result<Option<AuthSecretKey>, JsValue> {
        let filename = hash_pub_key(pub_key);

        let file: JsValue = JsFuture::from(self.keys_directory.get_file_handle(&filename)).await?;
        if file.is_null() {
            return Ok(None);
        }
        let file_handle = FileSystemFileHandle::from(file);
        let blob: web_sys::Blob = JsFuture::from(file_handle.get_file()).await?.into();

        let buffer = ArrayBuffer::unchecked_from_js(JsFuture::from(blob.array_buffer()).await?);
        let uint8_array = Uint8Array::new(&buffer);
        let mut secret_key_hex = vec![0; blob.size() as usize];
        uint8_array.copy_to(&mut secret_key_hex);

        let secret_key_bytes = hex::decode(secret_key_hex).map_err(|err| {
            JsValue::from_str(format!("error decoding secret key hex: {err:?}").as_str())
        })?;

        let secret_key = AuthSecretKey::read_from_bytes(&secret_key_bytes).map_err(|err| {
            JsValue::from_str(format!("error reading secret key: {err:?}").as_str())
        })?;

        Ok(Some(secret_key))
    }
}

impl<R: Rng> TransactionAuthenticator for WebKeyStore<R> {
    /// Gets a signature over a message, given a public key.
    ///
    /// The public key should correspond to one of the keys tracked by the keystore.
    ///
    /// # Errors
    /// If the public key isn't found in the store, [`AuthenticationError::UnknownPublicKey`] is
    /// returned.
    fn get_signature(
        &self,
        pub_key: Word,
        message: Word,
        _account_delta: &AccountDelta,
    ) -> Result<Vec<Felt>, AuthenticationError> {
        let mut rng = self.rng.write();
        let secret_key = self
            .get_key(pub_key)
            .block_on()
            .map_err(|err| AuthenticationError::other(format!("{err:#?}")))?;
        let AuthSecretKey::RpoFalcon512(k) = secret_key
            .ok_or(AuthenticationError::UnknownPublicKey(Digest::from(pub_key).into()))?;
        miden_tx::auth::signatures::get_falcon_signature(&k, message, &mut *rng)
    }
}
