use miden_client::note::{
    NetworkAccountTarget as NativeNetworkAccountTarget,
    NoteAttachment as NativeNoteAttachment,
};
use wasm_bindgen::prelude::*;

use super::account_id::AccountId;
use super::note_attachment::NoteAttachment;
use super::note_execution_hint::NoteExecutionHint;
use crate::js_error_with_context;

/// A standard note attachment that indicates a note should be consumed by a
/// specific network account.
///
/// Network accounts are accounts whose storage mode is `Network`, meaning the
/// network (nodes) can execute transactions on behalf of the account.
#[derive(Clone)]
#[wasm_bindgen]
pub struct NetworkAccountTarget(NativeNetworkAccountTarget);

#[wasm_bindgen]
impl NetworkAccountTarget {
    /// Creates a new network account target attachment.
    ///
    /// # Arguments
    /// * `target_id` - The ID of the network account that should consume the note
    /// * `exec_hint` - A hint about when the note can be executed
    ///
    /// # Errors
    /// Returns an error if the target account is not a network account.
    #[wasm_bindgen(constructor)]
    pub fn new(
        target_id: &AccountId,
        exec_hint: &NoteExecutionHint,
    ) -> Result<NetworkAccountTarget, JsValue> {
        let native_target = NativeNetworkAccountTarget::new(target_id.into(), exec_hint.into())
            .map_err(|e| js_error_with_context(e, "Failed to create NetworkAccountTarget"))?;
        Ok(NetworkAccountTarget(native_target))
    }

    /// Returns the ID of the target network account.
    #[wasm_bindgen(js_name = "targetId")]
    pub fn target_id(&self) -> AccountId {
        self.0.target_id().into()
    }

    /// Converts this target into a note attachment.
    #[wasm_bindgen(js_name = "intoAttachment")]
    pub fn into_attachment(&self) -> NoteAttachment {
        let attachment: NativeNoteAttachment = self.0.clone().into();
        attachment.into()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNetworkAccountTarget> for NetworkAccountTarget {
    fn from(native: NativeNetworkAccountTarget) -> Self {
        NetworkAccountTarget(native)
    }
}

impl From<&NativeNetworkAccountTarget> for NetworkAccountTarget {
    fn from(native: &NativeNetworkAccountTarget) -> Self {
        NetworkAccountTarget(native.clone())
    }
}

impl From<NetworkAccountTarget> for NativeNetworkAccountTarget {
    fn from(target: NetworkAccountTarget) -> Self {
        target.0
    }
}

impl From<&NetworkAccountTarget> for NativeNetworkAccountTarget {
    fn from(target: &NetworkAccountTarget) -> Self {
        target.0.clone()
    }
}
