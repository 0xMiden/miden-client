//! This module defines common structs to be used within the [`Store`](crate::store::Store) for
//! notes that are available to be consumed ([`InputNoteRecord`]) and notes that have been produced
//! as a result of executing a transaction ([`OutputNoteRecord`]).
//!
//! Both structs are similar in terms of the data they carry, but are differentiated semantically
//! as they are involved in very different flows. As such, known states are modeled differently for
//! the two structures, with [`InputNoteRecord`] having states described by the [`InputNoteState`]
//! enum.
//!
//! ## Serialization / Deserialization
//!
//! We provide serialization and deserialization support via [`Serializable`] and [`Deserializable`]
//! traits implementations.
//!
//! ## Type conversion
//!
//! We also facilitate converting from/into [`InputNote`](miden_protocol::transaction::InputNote) /
//! [`Note`](miden_protocol::note::Note), although this is not always possible. Check both
//! [`InputNoteRecord`]'s and [`OutputNoteRecord`]'s documentation for more details about this.

use alloc::string::{String, ToString};

use miden_protocol::errors::NoteError;
use thiserror::Error;

use crate::errors::ErrorCode;

mod input_note_record;
mod output_note_record;

pub use input_note_record::{InputNoteRecord, InputNoteState};
pub use output_note_record::{NoteExportType, OutputNoteRecord, OutputNoteState};

/// Contains structures that model all states in which an input note can be.
pub mod input_note_states {
    pub use super::input_note_record::{
        CommittedNoteState,
        ConsumedAuthenticatedLocalNoteState,
        ExpectedNoteState,
        InputNoteState,
        InvalidNoteState,
        ProcessingAuthenticatedNoteState,
        ProcessingUnauthenticatedNoteState,
        UnverifiedNoteState,
    };
}

// NOTE RECORD ERROR
// ================================================================================================

/// Errors generated from note records.
#[derive(Debug, Error)]
pub enum NoteRecordError {
    /// Error generated during conversion of note record.
    #[error("note record conversion error: {0}")]
    ConversionError(String),
    /// Invalid underlying note object.
    #[error("note error")]
    NoteError(#[from] NoteError),
    /// Note record isn't consumable.
    #[error("note not consumable: {0}")]
    NoteNotConsumable(String),
    /// Invalid inclusion proof.
    #[error("invalid inclusion proof")]
    InvalidInclusionProof,
    /// Invalid state transition.
    #[error("invalid state transition: {0}")]
    InvalidStateTransition(String),
    /// Error generated during a state transition.
    #[error("state transition error: {0}")]
    StateTransitionError(String),
}

impl ErrorCode for NoteRecordError {
    fn error_code(&self) -> &'static str {
        match self {
            Self::ConversionError(_) => "MIDEN-NR-001",
            Self::NoteError(_) => "MIDEN-NR-002",
            Self::NoteNotConsumable(_) => "MIDEN-NR-003",
            Self::InvalidInclusionProof => "MIDEN-NR-004",
            Self::InvalidStateTransition(_) => "MIDEN-NR-005",
            Self::StateTransitionError(_) => "MIDEN-NR-006",
        }
    }
}

impl From<NoteRecordError> for String {
    fn from(err: NoteRecordError) -> String {
        err.to_string()
    }
}
