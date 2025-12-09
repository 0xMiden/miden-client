//! RPC Client for Web Applications
//!
//! This module provides a WebAssembly-compatible RPC client for interacting with Miden nodes.

use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_client::note::NoteId as NativeNoteId;
use miden_client::rpc::domain::note::FetchedNote as NativeFetchedNote;
use miden_client::rpc::{GrpcClient, NodeRpcClient};
use note::FetchedNote;
use wasm_bindgen::prelude::*;

use crate::js_error_with_context;
use crate::models::endpoint::Endpoint;
use crate::models::note_id::NoteId;
use crate::models::note_script::NoteScript;
use crate::models::word::Word;

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
    /// @param endpoint - Endpoint to connect to.
    #[wasm_bindgen(constructor)]
    pub fn new(endpoint: Endpoint) -> Result<RpcClient, JsValue> {
        let rpc_client = Arc::new(GrpcClient::new(&endpoint.into(), 0));

        Ok(RpcClient { inner: rpc_client })
    }

    /// Fetches notes by their IDs from the connected Miden node.
    ///
    /// @param note_ids - Array of [`NoteId`] objects to fetch
    /// @returns Promise that resolves to different data depending on the note type:
    /// - Private notes: Returns the `noteHeader`, and the  `inclusionProof`. The `note` field will
    ///   be `null`.
    /// - Public notes: Returns the full `note` with `inclusionProof`, alongside its header.
    #[allow(clippy::doc_markdown)]
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
                NativeFetchedNote::Private(header, inclusion_proof) => {
                    FetchedNote::from_header(header, None, inclusion_proof)
                },
                NativeFetchedNote::Public(note, inclusion_proof) => {
                    let header = miden_client::note::NoteHeader::new(note.id(), *note.metadata());
                    FetchedNote::from_header(header, Some(note.into()), inclusion_proof)
                },
            })
            .collect();

        Ok(web_notes)
    }

    /// Fetches a note script by its root hash from the connected Miden node.
    ///
    /// @param script_root - The root hash of the note script to fetch.
    /// @returns Promise that resolves to the `NoteScript`.
    #[allow(clippy::doc_markdown)]
    #[wasm_bindgen(js_name = "getNoteScriptByRoot")]
    pub async fn get_note_script_by_root(&self, script_root: Word) -> Result<NoteScript, JsValue> {
        let native_script_root = script_root.into();

        let note_script = self
            .inner
            .get_note_script_by_root(native_script_root)
            .await
            .map_err(|err| js_error_with_context(err, "failed to get note script by root"))?;

        Ok(note_script.into())
    }
}
