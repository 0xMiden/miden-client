//! RPC Client for Web Applications
//!
//! This module provides a WebAssembly-compatible RPC client for interacting with Miden nodes.

use alloc::collections::BTreeSet;
use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_client::block::BlockNumber;
use miden_client::note::{NoteId as NativeNoteId, Nullifier};
use miden_client::rpc::domain::account::AccountStorageRequirements as NativeAccountStorageRequirements;
use miden_client::rpc::domain::note::FetchedNote as NativeFetchedNote;
use miden_client::rpc::{AccountStateAt, GrpcClient, NodeRpcClient};
use note::FetchedNote;
use wasm_bindgen::prelude::*;

use crate::js_error_with_context;
use crate::models::account_id::AccountId;
use crate::models::account_proof::AccountProof;
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
                    let header =
                        miden_client::note::NoteHeader::new(note.id(), note.metadata().clone());
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

    /// Fetches a block header by number. When `block_num` is undefined, returns the latest header.
    #[wasm_bindgen(js_name = "getBlockHeaderByNumber")]
    pub async fn get_block_header_by_number(
        &self,
        block_num: Option<u32>,
    ) -> Result<BlockHeader, JsValue> {
        let native_block_num = block_num.map(BlockNumber::from);
        let (header, _proof) =
            self.inner.get_block_header_by_number(native_block_num, false).await.map_err(
                |err| js_error_with_context(err, "failed to get block header by number"),
            )?;

        Ok(header.into())
    }

    /// Fetches account details for a specific account ID.
    #[wasm_bindgen(js_name = "getAccountDetails")]
    pub async fn get_account_details(
        &self,
        account_id: &AccountId,
    ) -> Result<FetchedAccount, JsValue> {
        let fetched = self
            .inner
            .get_account_details(account_id.into())
            .await
            .map_err(|err| js_error_with_context(err, "failed to get account details"))?;

        Ok(fetched.into())
    }

    /// Fetches an account proof from the node.
    ///
    /// This is a lighter-weight alternative to `getAccountDetails` that makes a single RPC call
    /// and returns the account proof and, for public accounts, the account header, storage slot
    /// values, and account code without reconstructing the full account state.
    ///
    /// For private accounts, the proof is returned but account details will not be available
    /// since they are not stored on-chain.
    ///
    /// Useful for reading storage slot values (e.g., faucet metadata) without the overhead of
    /// fetching the complete account with all vault assets and storage map entries.
    #[wasm_bindgen(js_name = "getAccountProof")]
    pub async fn get_account_proof(&self, account_id: &AccountId) -> Result<AccountProof, JsValue> {
        let native_id: miden_client::account::AccountId = account_id.into();

        let (block_num, proof) = self
            .inner
            .get_account_proof(
                native_id,
                NativeAccountStorageRequirements::default(),
                AccountStateAt::ChainTip,
                None,
            )
            .await
            .map_err(|err| js_error_with_context(err, "failed to get account"))?;

        Ok(AccountProof::new(proof, block_num))
    }

    /// Fetches notes matching the provided tags from the node.
    #[wasm_bindgen(js_name = "syncNotes")]
    pub async fn sync_notes(
        &self,
        block_num: u32,
        block_to: Option<u32>,
        note_tags: Vec<NoteTag>,
    ) -> Result<NoteSyncInfo, JsValue> {
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
            .map_err(|err| js_error_with_context(err, "failed to sync notes"))?;

        Ok(info.into())
    }

    // TODO: This can be generalized to retrieve multiple nullifiers
    /// Fetches the block height at which a nullifier was committed, if any.
    #[wasm_bindgen(js_name = "getNullifierCommitHeight")]
    pub async fn get_nullifier_commit_height(
        &self,
        nullifier: Word,
        block_num: u32,
    ) -> Result<Option<u32>, JsValue> {
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
            .map_err(|err| js_error_with_context(err, "failed to get nullifier commit height"))?
            .into_iter()
            .next()
            .and_then(|(_, height)| height);

        Ok(height.map(|height| height.as_u32()))
    }
}
