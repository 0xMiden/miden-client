use js_export_macro::js_export;
use miden_client::auth::Signature as NativeSignature;

use crate::models::felt::Felt;
use crate::models::word::Word;
use crate::platform::{JsBytes, JsErr};
use crate::utils::{deserialize_from_bytes, serialize_to_bytes};

/// Cryptographic signature produced by supported auth schemes.
#[js_export]
#[derive(Clone)]
pub struct Signature(NativeSignature);

#[js_export]
impl Signature {
    /// Serializes the signature into bytes.
    pub fn serialize(&self) -> JsBytes {
        serialize_to_bytes(&self.0)
    }

    /// Deserializes a signature from bytes.
    pub fn deserialize(bytes: JsBytes) -> Result<Signature, JsErr> {
        let native_signature = deserialize_from_bytes::<NativeSignature>(&bytes)?;
        Ok(Signature(native_signature))
    }

    /// Converts the signature to the prepared field elements expected by verifying code.
    #[js_export(js_name = "toPreparedSignature")]
    pub fn to_prepared_signature(&self, message: Word) -> Vec<Felt> {
        self.0
            .to_prepared_signature(message.into())
            .into_iter()
            .map(Into::into)
            .collect()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeSignature> for Signature {
    fn from(native_signature: NativeSignature) -> Self {
        Signature(native_signature)
    }
}

impl From<&NativeSignature> for Signature {
    fn from(native_signature: &NativeSignature) -> Self {
        Signature(native_signature.clone())
    }
}

impl From<Signature> for NativeSignature {
    fn from(signature: Signature) -> Self {
        signature.0
    }
}

impl From<&Signature> for NativeSignature {
    fn from(signature: &Signature) -> Self {
        signature.0.clone()
    }
}
