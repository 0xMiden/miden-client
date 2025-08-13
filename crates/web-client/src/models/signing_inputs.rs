use miden_objects::crypto::dsa::rpo_falcon512::{
    PublicKey as NativePublicKey,
    SecretKey as NativeSecretKey,
};
use miden_tx::auth::SigningInputs as NativeSigningInputs;
use rand::SeedableRng;
use rand::rngs::StdRng;
use wasm_bindgen::prelude::*;

use crate::models::felt::Felt;
use crate::models::public_key::PublicKey;
use crate::models::secret_key::SecretKey;
use crate::models::signature::Signature;
use crate::models::transaction_summary::TransactionSummary;
use crate::models::word::Word;

#[wasm_bindgen]
pub struct SigningInputs {
    inner: NativeSigningInputs,
}

#[wasm_bindgen]
impl SigningInputs {
    #[wasm_bindgen(js_name = "newTransactionSummary")]
    pub fn new_transaction_summary(summary: TransactionSummary) -> Self {
        Self {
            inner: NativeSigningInputs::TransactionSummary(Box::new(summary.into())),
        }
    }

    #[wasm_bindgen(js_name = "newArbitrary")]
    pub fn new_arbitrary(felts: Vec<Felt>) -> Self {
        Self {
            inner: NativeSigningInputs::Arbitrary(felts.into_iter().map(Into::into).collect()),
        }
    }

    #[wasm_bindgen(js_name = "newBlind")]
    pub fn new_blind(word: Word) -> Self {
        Self {
            inner: NativeSigningInputs::Blind(word.into()),
        }
    }

    #[wasm_bindgen(getter, js_name = "variantType")]
    pub fn variant_type(&self) -> String {
        match &self.inner {
            NativeSigningInputs::TransactionSummary(_) => "TransactionSummary".to_string(),
            NativeSigningInputs::Arbitrary(_) => "Arbitrary".to_string(),
            NativeSigningInputs::Blind(_) => "Blind".to_string(),
        }
    }

    #[wasm_bindgen(js_name = "toCommitment")]
    pub fn to_commitment(&self) -> Word {
        self.inner.to_commitment().into()
    }

    #[wasm_bindgen(js_name = "toElements")]
    pub fn to_elements(&self) -> Vec<Felt> {
        self.inner.to_elements().into_iter().map(Into::into).collect()
    }

    #[wasm_bindgen]
    pub fn sign(&self, secret_key: &SecretKey) -> Signature {
        let mut rng = StdRng::from_os_rng();
        let native_secret_key: NativeSecretKey = secret_key.into();
        let native_word = self.inner.to_commitment();
        native_secret_key.sign_with_rng(native_word, &mut rng).into()
    }

    #[wasm_bindgen]
    pub fn verify(&self, public_key: &PublicKey, signature: &Signature) -> bool {
        let native_public_key: NativePublicKey = public_key.into();
        let native_signature = signature.into();
        let native_word = self.inner.to_commitment();
        native_public_key.verify(native_word, &native_signature)
    }
}
