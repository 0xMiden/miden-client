use miden_client::store::NoteFilter as NativeNoteFilter;
use crate::prelude::*;

use super::note_id::NoteId;

// TODO: Add nullifier support

#[cfg(feature = "wasm")]
mod wasm_def {
    use wasm_bindgen::prelude::*;

    /// Filter type options for querying notes from the store.
    #[derive(Clone)]
    #[wasm_bindgen]
    pub enum NoteFilterTypes {
        All,
        Consumed,
        Committed,
        Expected,
        Processing,
        List,
        Unique,
        Nullifiers,
        Unverified,
    }
}

#[cfg(feature = "napi")]
mod napi_def {
    use napi_derive::napi;

    /// Filter type options for querying notes from the store.
    #[napi(string_enum)]
    pub enum NoteFilterTypes {
        All,
        Consumed,
        Committed,
        Expected,
        Processing,
        List,
        Unique,
        Nullifiers,
        Unverified,
    }
}

#[cfg(feature = "wasm")]
pub use wasm_def::NoteFilterTypes;

#[cfg(feature = "napi")]
pub use napi_def::NoteFilterTypes;

/// Filter options for querying notes from the store.
#[bindings]
#[derive(Clone)]
pub struct NoteFilter {
    note_type: NoteFilterTypes,
    note_ids: Option<Vec<NoteId>>,
}

// Platform-specific constructors that differ in signature
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl NoteFilter {
    #[wasm_bindgen(constructor)]
    pub fn new(note_type: NoteFilterTypes, note_ids: Option<Vec<NoteId>>) -> NoteFilter {
        NoteFilter { note_type, note_ids }
    }
}

#[cfg(feature = "napi")]
#[napi_derive::napi]
impl NoteFilter {
    #[napi(constructor)]
    pub fn new(note_type: NoteFilterTypes, note_ids: Option<Vec<&NoteId>>) -> NoteFilter {
        NoteFilter {
            note_type,
            note_ids: note_ids.map(|ids| ids.into_iter().map(|id| *id).collect()),
        }
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NoteFilter> for NativeNoteFilter {
    fn from(filter: NoteFilter) -> Self {
        match filter.note_type {
            NoteFilterTypes::All => NativeNoteFilter::All,
            NoteFilterTypes::Consumed => NativeNoteFilter::Consumed,
            NoteFilterTypes::Committed => NativeNoteFilter::Committed,
            NoteFilterTypes::Expected => NativeNoteFilter::Expected,
            NoteFilterTypes::Processing => NativeNoteFilter::Processing,
            NoteFilterTypes::List => {
                let note_ids =
                    filter.note_ids.unwrap_or_else(|| panic!("Note IDs required for List filter"));
                NativeNoteFilter::List(note_ids.iter().map(Into::into).collect())
            },
            NoteFilterTypes::Unique => {
                let note_ids =
                    filter.note_ids.unwrap_or_else(|| panic!("Note ID required for Unique filter"));

                assert!(note_ids.len() == 1, "Only one Note ID can be provided");

                NativeNoteFilter::Unique(note_ids.first().unwrap().into())
            },
            NoteFilterTypes::Nullifiers => NativeNoteFilter::Nullifiers(vec![]),
            NoteFilterTypes::Unverified => NativeNoteFilter::Unverified,
        }
    }
}

impl From<&NoteFilter> for NativeNoteFilter {
    fn from(filter: &NoteFilter) -> Self {
        match filter.note_type {
            NoteFilterTypes::All => NativeNoteFilter::All,
            NoteFilterTypes::Consumed => NativeNoteFilter::Consumed,
            NoteFilterTypes::Committed => NativeNoteFilter::Committed,
            NoteFilterTypes::Expected => NativeNoteFilter::Expected,
            NoteFilterTypes::Processing => NativeNoteFilter::Processing,
            NoteFilterTypes::List => {
                let note_ids = filter
                    .note_ids
                    .clone()
                    .unwrap_or_else(|| panic!("Note IDs required for List filter"));
                NativeNoteFilter::List(note_ids.iter().map(Into::into).collect())
            },
            NoteFilterTypes::Unique => {
                let note_ids = filter
                    .note_ids
                    .clone()
                    .unwrap_or_else(|| panic!("Note ID required for Unique filter"));

                assert!(note_ids.len() == 1, "Only one Note ID can be provided");

                NativeNoteFilter::Unique(note_ids.first().unwrap().into())
            },
            NoteFilterTypes::Nullifiers => NativeNoteFilter::Nullifiers(vec![]),
            NoteFilterTypes::Unverified => NativeNoteFilter::Unverified,
        }
    }
}
