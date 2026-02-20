//! RPC Client for Web and Node.js Applications
//!
//! This module provides a WebAssembly-compatible and napi-rs compatible RPC client
//! for interacting with Miden nodes.

use alloc::collections::BTreeSet;

use miden_client::block::BlockNumber;
use miden_client::note::{NoteId as NativeNoteId, Nullifier};
use miden_client::rpc::domain::note::FetchedNote as NativeFetchedNote;
use miden_client::rpc::{GrpcClient, NodeRpcClient};
use note::FetchedNote;

use crate::prelude::*;
use crate::models::account_id::AccountId;
use crate::models::block_header::BlockHeader;
use crate::models::endpoint::Endpoint;
use crate::models::fetched_account::FetchedAccount;
use crate::models::note_id::NoteId;
use crate::models::note_script::NoteScript;
use crate::models::note_sync_info::NoteSyncInfo;
use crate::models::note_tag::NoteTag;
use crate::models::word::Word;

mod note;

/// RPC Client for interacting with Miden nodes directly.
#[bindings]
pub struct RpcClient {
    inner: Arc<dyn NodeRpcClient>,
}

#[bindings]
impl RpcClient {
    /// Creates a new RPC client instance.
    ///
    /// @param endpoint - Endpoint to connect to.
    #[bindings(constructor)]
    pub fn new(endpoint: Endpoint) -> platform::JsResult<RpcClient> {
        #[cfg(feature = "wasm")]
        let retry_count = 0;
        #[cfg(feature = "napi")]
        let retry_count = 10_000;

        let rpc_client = Arc::new(GrpcClient::new(&endpoint.into(), retry_count));

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
    #[bindings(js_name = "getNotesById")]
    pub async fn get_notes_by_id(
        &self,
        note_ids: Vec<NoteId>,
    ) -> platform::JsResult<Vec<FetchedNote>> {
        let native_note_ids: Vec<NativeNoteId> =
            note_ids.into_iter().map(NativeNoteId::from).collect();

        let fetched_notes = self
            .inner
            .get_notes_by_id(&native_note_ids)
            .await
            .map_err(|err| platform::error_with_context(err, "failed to get notes by ID"))?;

        let notes: Vec<FetchedNote> = fetched_notes
            .into_iter()
            .map(|native_note| match native_note {
                NativeFetchedNote::Private(header, inclusion_proof) => {
                    FetchedNote::from_header(header, None, inclusion_proof)
                },
                NativeFetchedNote::Public(note, inclusion_proof) => {
                    let header =
                        miden_client::note::NoteHeader::new(note.id(), note.metadata().clone());
                    FetchedNote::from_header(header, Some(note.into()), inclusion_proof)
                },
            })
            .collect();

        Ok(notes)
    }

    /// Fetches a note script by its root hash from the connected Miden node.
    ///
    /// @param script_root - The root hash of the note script to fetch.
    /// @returns Promise that resolves to the `NoteScript`.
    #[allow(clippy::doc_markdown)]
    #[bindings(js_name = "getNoteScriptByRoot")]
    pub async fn get_note_script_by_root(&self, script_root: Word) -> platform::JsResult<NoteScript> {
        let native_script_root = script_root.into();

        let note_script = self
            .inner
            .get_note_script_by_root(native_script_root)
            .await
            .map_err(|err| platform::error_with_context(err, "failed to get note script by root"))?;

        Ok(note_script.into())
    }

    /// Fetches a block header by number. When `block_num` is undefined, returns the latest header.
    #[bindings(js_name = "getBlockHeaderByNumber")]
    pub async fn get_block_header_by_number(
        &self,
        block_num: Option<u32>,
    ) -> platform::JsResult<BlockHeader> {
        let native_block_num = block_num.map(BlockNumber::from);
        let (header, _proof) =
            self.inner.get_block_header_by_number(native_block_num, false).await.map_err(
                |err| platform::error_with_context(err, "failed to get block header by number"),
            )?;

        Ok(header.into())
    }

    /// Fetches account details for a specific account ID.
    #[bindings(js_name = "getAccountDetails")]
    pub async fn get_account_details(
        &self,
        account_id: AccountId,
    ) -> platform::JsResult<FetchedAccount> {
        let fetched = self
            .inner
            .get_account_details(account_id.into())
            .await
            .map_err(|err| platform::error_with_context(err, "failed to get account details"))?;

        Ok(fetched.into())
    }

    /// Fetches notes matching the provided tags from the node.
    #[bindings(js_name = "syncNotes")]
    pub async fn sync_notes(
        &self,
        block_num: u32,
        block_to: Option<u32>,
        note_tags: Vec<NoteTag>,
    ) -> platform::JsResult<NoteSyncInfo> {
        let mut tags = BTreeSet::new();
        for tag in note_tags {
            tags.insert(tag.into());
        }

        let block_num = BlockNumber::from(block_num);
        let block_to = block_to.map(BlockNumber::from);

        let info = self
            .inner
            .sync_notes(block_num, block_to, &tags)
            .await
            .map_err(|err| platform::error_with_context(err, "failed to sync notes"))?;

        Ok(info.into())
    }

    // TODO: This can be generalized to retrieve multiple nullifiers
    /// Fetches the block height at which a nullifier was committed, if any.
    #[bindings(js_name = "getNullifierCommitHeight")]
    pub async fn get_nullifier_commit_height(
        &self,
        nullifier: Word,
        block_num: u32,
    ) -> platform::JsResult<Option<u32>> {
        let native_word: miden_client::Word = nullifier.into();
        // TODO: nullifier JS binding
        let nullifier = Nullifier::from_raw(native_word);
        let block_num = BlockNumber::from(block_num);

        let mut requested_nullifiers = BTreeSet::new();
        requested_nullifiers.insert(nullifier);

        let height = self
            .inner
            .get_nullifier_commit_heights(requested_nullifiers, block_num)
            .await
            .map_err(|err| platform::error_with_context(err, "failed to get nullifier commit height"))?
            .into_iter()
            .next()
            .and_then(|(_, height)| height);

        Ok(height.map(|height| height.as_u32()))
    }
}
