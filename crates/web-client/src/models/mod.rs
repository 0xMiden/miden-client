//!# Module Overview
//!
//! This module provides a set of structs and functionality that are exposed to JavaScript via
//! `wasm_bindgen`. These structs serve as wrappers around native objects from the acrss the miden
//! repositories. The goal is to provide a way to interact with these objects in a web context with
//! JavaScript, mimicking the same level of functionality and usability as when working with them in
//! Rust.
//!
//! ## Purpose
//!
//! This module is designed to enable developers to work with core objects and data structures used
//! in the miden client, directly from JavaScript in a browser environment. By exposing Rust-native
//! functionality via `wasm_bindgen`, it ensures that the web-based use of the miden client is as
//! close as possible to the Rust-native experience. These bindings allow the creation and
//! manipulation of important client structures, such as accounts, transactions, notes, and assets,
//! providing access to core methods and properties.
//!
//! ## Usage
//!
//! Each module provides Rust structs and methods that are exposed to JavaScript via `wasm_bindgen`.
//! These bindings allow developers to create and manipulate miden client objects in JavaScript,
//! while maintaining the same functionality and control as would be available in a pure Rust
//! environment.
//!
//! This makes it easy to build web-based applications that interact with the miden client, enabling
//! rich interaction with accounts, assets, and transactions directly from the browser.

#![allow(clippy::return_self_not_must_use)]

pub mod account;
pub mod account_builder;
pub mod account_code;
pub mod account_component;
pub mod account_delta;
pub mod account_header;
pub mod account_id;
pub mod account_storage;
pub mod account_storage_mode;
pub mod account_storage_requirements;
pub mod account_type;
pub mod address;
pub mod advice_inputs;
pub mod advice_map;
pub mod assembler;
pub mod asset_vault;
pub mod auth_secret_key;
pub mod basic_fungible_faucet_component;
pub mod block_header;
pub mod consumable_note_record;
pub mod endpoint;
pub mod executed_transaction;
pub mod felt;
pub mod foreign_account;
pub mod fungible_asset;
pub mod input_note;
pub mod input_note_record;
pub mod input_note_state;
pub mod input_notes;
pub mod library;
pub mod merkle_path;
pub mod note;
pub mod note_assets;
pub mod note_details;
pub mod note_execution_hint;
pub mod note_execution_mode;
pub mod note_filter;
pub mod note_header;
pub mod note_id;
pub mod note_inclusion_proof;
pub mod note_inputs;
pub mod note_location;
pub mod note_metadata;
pub mod note_recipient;
pub mod note_script;
pub mod note_tag;
pub mod note_type;
pub mod output_note;
pub mod output_notes;
pub mod partial_note;
pub mod provers;
pub mod public_key;
pub mod rpo256;
pub mod secret_key;
pub mod signature;
pub mod signing_inputs;
pub mod storage_map;
pub mod storage_slot;
pub mod sync_summary;
pub mod token_symbol;
pub mod transaction_args;
pub mod transaction_filter;
pub mod transaction_id;
pub mod transaction_kernel;
pub mod transaction_record;
pub mod transaction_request;
pub mod transaction_result;
pub mod transaction_script;
pub mod transaction_script_inputs;
pub mod transaction_status;
pub mod transaction_summary;
pub mod word;

declare_js_miden_arrays! {
    (crate::models::account::Account) -> AccountArray,
    (crate::models::account_builder::AccountBuilder) -> AccountBuilderArray,
    (crate::models::account_code::AccountCode) -> AccountCodeArray,
    (crate::models::account_component::AccountComponent) -> AccountComponentArray,
    (crate::models::account_delta::AccountDelta) -> AccountDeltaArray,
    (crate::models::account_header::AccountHeader) -> AccountHeaderArray,
    (crate::models::account_id::AccountId) -> AccountIdArray,
    (crate::models::account_storage::AccountStorage) -> AccountStorageArray,
    (crate::models::account_storage_mode::AccountStorageMode) -> AccountStorageModeArray,
    (crate::models::account_storage_requirements::AccountStorageRequirements) -> AccountStorageRequirementsArray,
    (crate::models::account_type::AccountType) -> AccountTypeArray,
    (crate::models::address::Address) -> AddressArray,
    (crate::models::advice_inputs::AdviceInputs) -> AdviceInputsArray,
    (crate::models::advice_map::AdviceMap) -> AdviceMapArray,
    (crate::models::assembler::Assembler) -> AssemblerArray,
    (crate::models::asset_vault::AssetVault) -> AssetVaultArray,
    (crate::models::auth_secret_key::AuthSecretKey) -> AuthSecretKeyArray,
    (crate::models::block_header::BlockHeader) -> BlockHeaderArray,
    (crate::models::consumable_note_record::ConsumableNoteRecord) -> ConsumableNoteRecordArray,
    (crate::models::endpoint::Endpoint) -> EndpointArray,
    (crate::models::executed_transaction::ExecutedTransaction) -> ExecutedTransactionArray,
    (crate::models::foreign_account::ForeignAccount) -> ForeignAccountArray,
    (crate::models::fungible_asset::FungibleAsset) -> FungibleAssetArray,
    (crate::models::input_note::InputNote) -> InputNoteArray,
    (crate::models::input_note_record::InputNoteRecord) -> InputNoteRecordArray,
    (crate::models::input_note_state::InputNoteState) -> InputNoteStateArray,
    (crate::models::input_notes::InputNotes) -> InputNotesArray,
    (crate::models::library::Library) -> LibraryArray,
    (crate::models::merkle_path::MerklePath) -> MerklePathArray,
    (crate::models::note::Note) -> NoteArray,
    (crate::models::note_assets::NoteAssets) -> NoteAssetsArray,
    (crate::models::note_execution_hint::NoteExecutionHint) -> NoteExecutionHintArray,
    (crate::models::note_execution_mode::NoteExecutionMode) -> NoteExecutionModeArray,
    (crate::models::note_filter::NoteFilter) -> NoteFilterArray,
    (crate::models::note_header::NoteHeader) -> NoteHeaderArray,
    (crate::models::note_id::NoteId) -> NoteIdArray,
    (crate::models::note_inclusion_proof::NoteInclusionProof) -> NoteInclusionProofArray,
    (crate::models::note_inputs::NoteInputs) -> NoteInputsArray,
    (crate::models::note_location::NoteLocation) -> NoteLocationArray,
    (crate::models::note_metadata::NoteMetadata) -> NoteMetadataArray,
    (crate::models::note_recipient::NoteRecipient) -> NoteRecipientArray,
    (crate::models::note_script::NoteScript) -> NoteScriptArray,
    (crate::models::note_tag::NoteTag) -> NoteTagArray,
    (crate::models::note_type::NoteType) -> NoteTypeArray,
    (crate::models::output_note::OutputNote) -> OutputNoteArray,
    (crate::models::partial_note::PartialNote) -> PartialNoteArray,
    (crate::models::provers::TransactionProver) -> TransactionProverArray,
    (crate::models::public_key::PublicKey) -> PublicKeyArray,
    (crate::models::rpo256::Rpo256) -> Rpo256Array,
    (crate::models::secret_key::SecretKey) -> SecretKeyArray,
    (crate::models::signature::Signature) -> SignatureArray,
    (crate::models::signing_inputs::SigningInputs) -> SigningInputsArray,
    (crate::models::storage_map::StorageMap) -> StorageMapArray,
    (crate::models::storage_slot::StorageSlot) -> StorageSlotArray,
    (crate::models::token_symbol::TokenSymbol) -> TokenSymbolArray,
    (crate::models::transaction_args::TransactionArgs) -> TransactionArgsArray,
    (crate::models::transaction_filter::TransactionFilter) -> TransactionFilterArray,
    (crate::models::transaction_id::TransactionId) -> TransactionIdArray,
    (crate::models::transaction_record::TransactionRecord) -> TransactionRecordArray,
    (crate::models::transaction_request::TransactionRequest) -> TransactionRequestArray,
    (crate::models::transaction_result::TransactionResult) -> TransactionResultArray,
    (crate::models::transaction_script::TransactionScript) -> TransactionScriptArray,
    (crate::models::transaction_script_inputs::TransactionScriptInputPair) -> TransactionScriptInputPairArray,
    (crate::models::transaction_status::TransactionStatus) -> TransactionStatusArray,
    (crate::models::transaction_summary::TransactionSummary) -> TransactionSummaryArray,
    (crate::models::word::Word) -> WordArray,
    (crate::models::felt::Felt) -> FeltArray,
    (crate::models::output_notes::OutputNotes) -> OutputNotesArray,
    (crate::models::note_details::NoteDetails) -> NoteDetailsArray,
    (crate::models::transaction_request::note_and_args::NoteAndArgs) -> NoteAndArgsArray,
    (crate::models::transaction_request::note_details_and_tag::NoteDetailsAndTag) -> NoteDetailsAndTagArray,
    (crate::models::transaction_request::note_id_and_args::NoteIdAndArgs) -> NoteIdAndArgsArray
    // FIXME: Types that need clone
    // (crate::models::transaction_kernel::TransactionKernel) -> TransactionKernelArray,
    // (crate::models::basic_fungible_faucet_component::BasicFungibleFaucetComponent) -> BasicFungibleFaucetComponentArray,
    // (crate::models::sync_summary::SyncSummary) -> SyncSummaryArray,
}
// }
