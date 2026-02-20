use miden_client::Word as NativeWord;
use miden_client::account::component::{
    AccountComponent as NativeAccountComponent,
    AuthFalcon512RpoMultisigConfig as NativeAuthFalcon512RpoMultisigConfig,
    falcon_512_rpo_multisig_library,
};
use miden_client::account::{
    StorageMap as NativeStorageMap,
    StorageSlot as NativeStorageSlot,
    StorageSlotName,
};
use miden_client::auth::PublicKeyCommitment;
use crate::prelude::*;

use crate::models::account_component::AccountComponent;
use crate::models::word::Word;
use crate::platform;

#[bindings]
#[derive(Clone)]
pub struct ProcedureThreshold {
    proc_root: Word,
    threshold: u32,
}

// Shared methods
#[bindings]
impl ProcedureThreshold {
    #[bindings(getter)]
    pub fn proc_root(&self) -> Word {
        self.proc_root.clone()
    }

    #[bindings(getter)]
    pub fn threshold(&self) -> u32 {
        self.threshold
    }
}

// wasm constructor
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl ProcedureThreshold {
    #[wasm_bindgen(constructor)]
    pub fn new(proc_root: &Word, threshold: u32) -> ProcedureThreshold {
        ProcedureThreshold { proc_root: proc_root.clone(), threshold }
    }
}

// napi constructor
#[cfg(feature = "napi")]
#[napi_derive::napi]
impl ProcedureThreshold {
    #[napi(constructor)]
    pub fn new(proc_root: &Word, threshold: u32) -> ProcedureThreshold {
        ProcedureThreshold {
            proc_root: proc_root.clone(),
            threshold,
        }
    }
}

/// Multisig auth configuration for `RpoFalcon512` signatures.
#[bindings]
#[derive(Clone)]
pub struct AuthFalcon512RpoMultisigConfig(NativeAuthFalcon512RpoMultisigConfig);

// Shared methods
#[bindings]
impl AuthFalcon512RpoMultisigConfig {
    #[bindings(getter)]
    pub fn default_threshold(&self) -> u32 {
        self.0.default_threshold()
    }

    /// Approver public key commitments as Words.
    #[bindings(getter)]
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
    #[bindings]
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

// wasm: constructor takes Vec<Word> (owned), with_proc_thresholds consumes self
#[cfg(feature = "wasm")]
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
    ) -> platform::JsResult<AuthFalcon512RpoMultisigConfig> {
        let native_approvers: Vec<PublicKeyCommitment> = approvers
            .into_iter()
            .map(|word| {
                let native_word: NativeWord = word.into();
                PublicKeyCommitment::from(native_word)
            })
            .collect();

        let config = NativeAuthFalcon512RpoMultisigConfig::new(native_approvers, default_threshold)
            .map_err(|e| platform::error_with_context(e, "Invalid multisig config"))?;

        Ok(AuthFalcon512RpoMultisigConfig(config))
    }

    /// Attach per-procedure thresholds. Each threshold must be >= 1 and <= `approvers.length`.
    
    pub fn with_proc_thresholds(
        self,
        proc_thresholds: Vec<ProcedureThreshold>,
    ) -> platform::JsResult<AuthFalcon512RpoMultisigConfig> {
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
            .map_err(|e| platform::error_with_context(e, "Invalid per-procedure thresholds"))?;

        Ok(AuthFalcon512RpoMultisigConfig(config))
    }
}

// napi: constructor takes Vec<&Word> (refs), with_proc_thresholds takes &self
#[cfg(feature = "napi")]
#[napi_derive::napi]
impl AuthFalcon512RpoMultisigConfig {
    /// Build a configuration with a list of approver public key commitments and a default
    /// threshold.
    ///
    /// `default_threshold` must be >= 1 and <= `approvers.length`.
    #[napi(constructor)]
    pub fn new(
        approvers: Vec<&Word>,
        default_threshold: u32,
    ) -> platform::JsResult<AuthFalcon512RpoMultisigConfig> {
        let native_approvers: Vec<PublicKeyCommitment> = approvers
            .into_iter()
            .map(|word| {
                let native_word: NativeWord = word.into();
                PublicKeyCommitment::from(native_word)
            })
            .collect();

        let config = NativeAuthFalcon512RpoMultisigConfig::new(native_approvers, default_threshold)
            .map_err(|e| platform::error_with_context(e, "Invalid multisig config"))?;

        Ok(AuthFalcon512RpoMultisigConfig(config))
    }

    /// Attach per-procedure thresholds. Each threshold must be >= 1 and <= `approvers.length`.
    pub fn with_proc_thresholds(
        &self,
        proc_thresholds: Vec<&ProcedureThreshold>,
    ) -> platform::JsResult<AuthFalcon512RpoMultisigConfig> {
        let native_proc_thresholds = proc_thresholds
            .into_iter()
            .map(|entry| {
                let proc_root: NativeWord = (&entry.proc_root).into();
                (proc_root, entry.threshold)
            })
            .collect();

        let config = self
            .0
            .clone()
            .with_proc_thresholds(native_proc_thresholds)
            .map_err(|e| platform::error_with_context(e, "Invalid per-procedure thresholds"))?;

        Ok(AuthFalcon512RpoMultisigConfig(config))
    }
}

/// Create an auth component for `Falcon512Rpo` multisig.
#[cfg(feature = "wasm")]
pub fn create_auth_falcon512_rpo_multisig(
    config: AuthFalcon512RpoMultisigConfig,
) -> platform::JsResult<AccountComponent> {
    let native_config: NativeAuthFalcon512RpoMultisigConfig = config.into();
    build_multisig_component(native_config)
}

/// Create an auth component for `Falcon512Rpo` multisig.
#[cfg(feature = "napi")]
pub fn create_auth_falcon512_rpo_multisig(
    config: &AuthFalcon512RpoMultisigConfig,
) -> platform::JsResult<AccountComponent> {
    let native_config: NativeAuthFalcon512RpoMultisigConfig = config.clone().into();
    build_multisig_component(native_config)
}

fn build_multisig_component(
    native_config: NativeAuthFalcon512RpoMultisigConfig,
) -> platform::JsResult<AccountComponent> {
    let mut storage_slots = Vec::with_capacity(4);

    let num_approvers = u32::try_from(native_config.approvers().len()).map_err(|e| {
        platform::error_with_context(e, "Too many approvers (would truncate num_approvers)")
    })?;

    // Slot 0: threshold_config
    let threshold_config_name =
        StorageSlotName::new("miden::standards::auth::falcon512_rpo_multisig::threshold_config")
            .map_err(|e| {
                platform::error_with_context(e, "Failed to create storage slot name 'threshold_config'")
            })?;
    storage_slots.push(NativeStorageSlot::with_value(
        threshold_config_name,
        NativeWord::from([native_config.default_threshold(), num_approvers, 0, 0]),
    ));

    let map_entries: Vec<_> = native_config
        .approvers()
        .iter()
        .enumerate()
        .map(|(i, pk)| {
            let idx = u32::try_from(i).map_err(|e| {
                platform::error_with_context(e, "Too many approvers (would truncate approver index)")
            })?;
            Ok((NativeWord::from([idx, 0, 0, 0]), (*pk).into()))
        })
        .collect::<Result<_, platform::PlatformError>>()?;
    let approver_map = NativeStorageMap::with_entries(map_entries.into_iter())
        .map_err(|e| platform::error_with_context(e, "Failed to build approver map"))?;
    let approver_map_name = StorageSlotName::new(
        "miden::standards::auth::falcon512_rpo_multisig::approver_public_keys",
    )
    .map_err(|e| {
        platform::error_with_context(e, "Failed to create storage slot name 'approver_public_keys'")
    })?;
    storage_slots.push(NativeStorageSlot::with_map(approver_map_name, approver_map));

    // Slot 2: executed_transactions map (empty)
    let executed_tx_map_name = StorageSlotName::new(
        "miden::standards::auth::falcon512_rpo_multisig::executed_transactions",
    )
    .map_err(|e| {
        platform::error_with_context(e, "Failed to create storage slot name 'executed_transactions'")
    })?;
    storage_slots
        .push(NativeStorageSlot::with_map(executed_tx_map_name, NativeStorageMap::default()));

    // Slot 3: procedure_thresholds map
    let proc_map_name = StorageSlotName::new(
        "miden::standards::auth::falcon512_rpo_multisig::procedure_thresholds",
    )
    .map_err(|e| {
        platform::error_with_context(e, "Failed to create storage slot name 'procedure_thresholds'")
    })?;
    let proc_map = NativeStorageMap::with_entries(
        native_config
            .proc_thresholds()
            .iter()
            .map(|(proc_root, threshold)| (*proc_root, NativeWord::from([*threshold, 0, 0, 0]))),
    )
    .map_err(|e| platform::error_with_context(e, "Failed to build proc thresholds map"))?;
    storage_slots.push(NativeStorageSlot::with_map(proc_map_name, proc_map));

    let native_component =
        NativeAccountComponent::new(falcon_512_rpo_multisig_library(), storage_slots)
            .map_err(|e| platform::error_with_context(e, "Failed to create multisig account component"))?
            .with_supports_all_types();

    Ok(native_component.into())
}

impl From<AuthFalcon512RpoMultisigConfig> for NativeAuthFalcon512RpoMultisigConfig {
    fn from(config: AuthFalcon512RpoMultisigConfig) -> Self {
        config.0
    }
}
