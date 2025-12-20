use miden_client::account::component::{
    AccountComponent as NativeAccountComponent,
    AuthRpoFalcon512MultisigConfig as NativeAuthRpoFalcon512MultisigConfig,
    rpo_falcon_512_multisig_library,
};
use miden_client::account::{StorageMap as NativeStorageMap, StorageSlot as NativeStorageSlot};
use miden_client::auth::PublicKeyCommitment;
use miden_client::Word as NativeWord;
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
        ProcedureThreshold {
            proc_root: proc_root.clone(),
            threshold,
        }
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
pub struct AuthRpoFalcon512MultisigConfig(NativeAuthRpoFalcon512MultisigConfig);

#[wasm_bindgen]
impl AuthRpoFalcon512MultisigConfig {
    /// Build a configuration with a list of approver public key commitments and a default threshold.
    ///
    /// `default_threshold` must be >= 1 and <= `approvers.length`.
    #[wasm_bindgen(constructor)]
    pub fn new(
        approvers: Vec<Word>,
        default_threshold: u32,
    ) -> Result<AuthRpoFalcon512MultisigConfig, JsValue> {
        let native_approvers: Vec<PublicKeyCommitment> = approvers
            .into_iter()
            .map(|word| {
                let native_word: NativeWord = word.into();
                PublicKeyCommitment::from(native_word)
            })
            .collect();

        let config =
            NativeAuthRpoFalcon512MultisigConfig::new(native_approvers, default_threshold).map_err(
                |e| js_error_with_context(e, "Invalid multisig config"),
            )?;

        Ok(AuthRpoFalcon512MultisigConfig(config))
    }

    /// Attach per-procedure thresholds. Each threshold must be >= 1 and <= `approvers.length`.
    #[wasm_bindgen(js_name = "withProcThresholds")]
    pub fn with_proc_thresholds(
        self,
        proc_thresholds: Vec<ProcedureThreshold>,
    ) -> Result<AuthRpoFalcon512MultisigConfig, JsValue> {
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

        Ok(AuthRpoFalcon512MultisigConfig(config))
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
            .map(|pkc| {
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

/// Create an auth component for `RpoFalcon512` multisig.
#[wasm_bindgen(js_name = "createAuthRpoFalcon512Multisig")]
pub fn create_auth_rpo_falcon512_multisig(
    config: AuthRpoFalcon512MultisigConfig,
) -> Result<AccountComponent, JsValue> {
    let native_config: NativeAuthRpoFalcon512MultisigConfig = config.into();

    // Build storage slots manually so we can surface errors instead of panicking.
    let mut storage_slots = Vec::with_capacity(4);

    // Slot 0: [threshold, num_approvers, 0, 0]
    let num_approvers = u32::try_from(native_config.approvers().len()).map_err(|e| {
        js_error_with_context(e, "Too many approvers (would truncate num_approvers)")
    })?;
    storage_slots.push(NativeStorageSlot::Value(NativeWord::from([
        native_config.default_threshold(),
        num_approvers,
        0,
        0,
    ])));

    // Slot 1: Approver public keys map
    let map_entries: Vec<_> = native_config
        .approvers()
        .iter()
        .enumerate()
        .map(|(i, pk)| {
            let idx = u32::try_from(i).map_err(|e| {
                js_error_with_context(e, "Too many approvers (would truncate approver index)")
            })?;
            Ok((NativeWord::from([idx, 0, 0, 0]), (*pk).into()))
        })
        .collect::<Result<_, JsValue>>()?;
    let approver_map = NativeStorageMap::with_entries(map_entries.into_iter())
        .map_err(|e| js_error_with_context(e, "Failed to build approver map"))?;
    storage_slots.push(NativeStorageSlot::Map(approver_map));

    // Slot 2: Executed transactions map (empty)
    storage_slots.push(NativeStorageSlot::Map(NativeStorageMap::default()));

    // Slot 3: Procedure thresholds map
    let proc_map = NativeStorageMap::with_entries(
        native_config
            .proc_thresholds()
            .iter()
            .map(|(proc_root, threshold)| (*proc_root, NativeWord::from([*threshold, 0, 0, 0]))),
    )
    .map_err(|e| js_error_with_context(e, "Failed to build proc thresholds map"))?;
    storage_slots.push(NativeStorageSlot::Map(proc_map));

    let native_component = NativeAccountComponent::new(rpo_falcon_512_multisig_library(), storage_slots)
        .map_err(|e| js_error_with_context(e, "Failed to create multisig account component"))?
        .with_supports_all_types();

    Ok(native_component.into())
}

impl From<AuthRpoFalcon512MultisigConfig> for NativeAuthRpoFalcon512MultisigConfig {
    fn from(config: AuthRpoFalcon512MultisigConfig) -> Self {
        config.0
    }
}
