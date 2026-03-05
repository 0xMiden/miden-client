use miden_client::Word as NativeWord;
use miden_client::auth::{
    AuthMultisig as NativeAuthMultisig,
    AuthMultisigConfig as NativeAuthMultisigConfig,
    AuthSchemeId as NativeAuthSchemeId,
    PublicKeyCommitment,
};
use wasm_bindgen::prelude::*;

use crate::js_error_with_context;
use crate::models::account_component::AccountComponent;
use crate::models::word::Word;

#[wasm_bindgen]
#[derive(Clone)]
pub struct ProcedureThreshold {
    proc_root: Word,
    threshold: u32,
}

#[wasm_bindgen]
impl ProcedureThreshold {
    #[wasm_bindgen(constructor)]
    pub fn new(proc_root: &Word, threshold: u32) -> ProcedureThreshold {
        ProcedureThreshold { proc_root: proc_root.clone(), threshold }
    }

    #[wasm_bindgen(getter, js_name = "procRoot")]
    pub fn proc_root(&self) -> Word {
        self.proc_root.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn threshold(&self) -> u32 {
        self.threshold
    }
}

/// Multisig auth configuration for `RpoFalcon512` signatures.
#[wasm_bindgen]
#[derive(Clone)]
pub struct AuthFalcon512RpoMultisigConfig(NativeAuthMultisigConfig);

#[wasm_bindgen]
impl AuthFalcon512RpoMultisigConfig {
    /// Build a configuration with a list of approver public key commitments and a default
    /// threshold.
    ///
    /// `default_threshold` must be >= 1 and <= `approvers.length`.
    #[wasm_bindgen(constructor)]
    pub fn new(
        approvers: Vec<Word>,
        default_threshold: u32,
    ) -> Result<AuthFalcon512RpoMultisigConfig, JsValue> {
        let native_approvers: Vec<(PublicKeyCommitment, NativeAuthSchemeId)> = approvers
            .into_iter()
            .map(|word| {
                let native_word: NativeWord = word.into();
                (PublicKeyCommitment::from(native_word), NativeAuthSchemeId::Falcon512Rpo)
            })
            .collect();

        let config = NativeAuthMultisigConfig::new(native_approvers, default_threshold)
            .map_err(|e| js_error_with_context(e, "Invalid multisig config"))?;

        Ok(AuthFalcon512RpoMultisigConfig(config))
    }

    /// Attach per-procedure thresholds. Each threshold must be >= 1 and <= `approvers.length`.
    #[wasm_bindgen(js_name = "withProcThresholds")]
    pub fn with_proc_thresholds(
        self,
        proc_thresholds: Vec<ProcedureThreshold>,
    ) -> Result<AuthFalcon512RpoMultisigConfig, JsValue> {
        let native_proc_thresholds = proc_thresholds
            .into_iter()
            .map(|entry| {
                let proc_root: NativeWord = entry.proc_root.into();
                (proc_root, entry.threshold)
            })
            .collect();

        let config = self
            .0
            .with_proc_thresholds(native_proc_thresholds)
            .map_err(|e| js_error_with_context(e, "Invalid per-procedure thresholds"))?;

        Ok(AuthFalcon512RpoMultisigConfig(config))
    }

    #[wasm_bindgen(getter, js_name = "defaultThreshold")]
    pub fn default_threshold(&self) -> u32 {
        self.0.default_threshold()
    }

    /// Approver public key commitments as Words.
    #[wasm_bindgen(getter)]
    pub fn approvers(&self) -> Vec<Word> {
        self.0
            .approvers()
            .iter()
            .map(|(pkc, _)| {
                let word: NativeWord = (*pkc).into();
                word.into()
            })
            .collect()
    }

    /// Per-procedure thresholds.
    #[wasm_bindgen(js_name = "getProcThresholds")]
    pub fn get_proc_thresholds(&self) -> Vec<ProcedureThreshold> {
        self.0
            .proc_thresholds()
            .iter()
            .map(|(proc_root, threshold)| ProcedureThreshold {
                proc_root: (*proc_root).into(),
                threshold: *threshold,
            })
            .collect()
    }
}

/// Create an auth component for `Falcon512Rpo` multisig.
#[wasm_bindgen(js_name = "createAuthFalcon512RpoMultisig")]
pub fn create_auth_falcon512_rpo_multisig(
    config: AuthFalcon512RpoMultisigConfig,
) -> Result<AccountComponent, JsValue> {
    let native_config: NativeAuthMultisigConfig = config.into();

    let multisig = NativeAuthMultisig::new(native_config)
        .map_err(|e| js_error_with_context(e, "Failed to create multisig auth component"))?;

    let native_component: miden_client::account::AccountComponent = multisig.into();

    Ok(native_component.into())
}

impl From<AuthFalcon512RpoMultisigConfig> for NativeAuthMultisigConfig {
    fn from(config: AuthFalcon512RpoMultisigConfig) -> Self {
        config.0
    }
}
