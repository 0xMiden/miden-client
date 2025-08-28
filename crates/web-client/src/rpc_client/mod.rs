//! RPC Client for Web Applications
//!
//! This module provides a WebAssembly-compatible RPC client for interacting with Miden nodes.

use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_client::rpc::domain::note::FetchedNote as NativeFetchedNote;
use miden_client::rpc::{NodeRpcClient, TonicRpcClient};
use miden_objects::note::NoteId as NativeNoteId;
use miden_objects::transaction::InputNote as NativeInputNote;
use note::FetchedNote;
use wasm_bindgen::prelude::*;

use crate::js_error_with_context;
use crate::models::endpoint::Endpoint;
use crate::models::note_id::NoteId;

mod note;

/// RPC Client for interacting with Miden nodes directly.
#[wasm_bindgen]
pub struct RpcClient {
    inner: Arc<dyn NodeRpcClient>,
}

#[wasm_bindgen]
impl RpcClient {
    /// Creates a new RPC client instance.
    ///
    /// @param endpoint: endpoint to connect to
    #[wasm_bindgen(constructor)]
    pub fn new(endpoint: Endpoint) -> Result<RpcClient, JsValue> {
        let rpc_client = Arc::new(TonicRpcClient::new(&endpoint.into(), 0));

        Ok(RpcClient { inner: rpc_client })
    }

    /// Fetches notes by their IDs from the connected Miden node.
    ///
    /// @param `note_ids` - Array of [`NoteId`] objects to fetch
    /// @returns Promise that resolves to  different data depending on the note type:
    /// - Private notes: Returns only `note_id` and `metadata`. The `input_note` field will be
    ///   `null`.
    /// - Public notes: Returns the full `input_note` with inclusion proof, alongside metadata and
    ///   ID.
    #[wasm_bindgen(js_name = "getNotesById")]
    pub async fn get_notes_by_id(
        &self,
        note_ids: Vec<NoteId>,
    ) -> Result<Vec<FetchedNote>, JsValue> {
        let native_note_ids: Vec<NativeNoteId> =
            note_ids.into_iter().map(NativeNoteId::from).collect();

        let fetched_notes = self
            .inner
            .get_notes_by_id(&native_note_ids)
            .await
            .map_err(|err| js_error_with_context(err, "failed to get notes by ID"))?;

        let web_notes: Vec<FetchedNote> = fetched_notes
            .into_iter()
            .map(|native_note| match native_note {
                NativeFetchedNote::Private(id, metadata, _inclusion_proof) => FetchedNote::new(
                    id.into(),
                    metadata.into(),
                    None, // Private notes don't include the full note
                ),
                NativeFetchedNote::Public(note, inclusion_proof) => {
                    let input_note = NativeInputNote::authenticated(note.clone(), inclusion_proof);

                    FetchedNote::new(
                        note.id().into(),
                        (*note.metadata()).into(),
                        Some(input_note.into()),
                    )
                },
            })
            .collect();

        Ok(web_notes)
    }
}
