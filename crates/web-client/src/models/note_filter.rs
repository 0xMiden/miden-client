use miden_client::store::NoteFilter as NativeNoteFilter;
use wasm_bindgen::prelude::*;

use super::note_id::NoteId;

// TODO: Add nullfiier support

/// Filters notes returned from the data store using type- or ID-based criteria.
#[derive(Clone)]
#[wasm_bindgen]
pub struct NoteFilter {
    note_type: NoteFilterTypes,
    note_ids: Option<Vec<NoteId>>,
}

#[wasm_bindgen]
impl NoteFilter {
    #[wasm_bindgen(constructor)]
    /// Creates a new filter from a filter type and optional note IDs.
    pub fn new(note_type: NoteFilterTypes, note_ids: Option<Vec<NoteId>>) -> NoteFilter {
        NoteFilter { note_type, note_ids }
    }
}

/// Enumerates the different note filter variants supported by the client.
#[derive(Clone)]
#[wasm_bindgen]
pub enum NoteFilterTypes {
    /// Return all notes.
    All,
    /// Only include notes that were consumed.
    Consumed,
    /// Only include notes that are committed.
    Committed,
    /// Only include notes that are expected.
    Expected,
    /// Only include notes currently being processed.
    Processing,
    /// Filter to a specific list of note IDs.
    List,
    /// Filter to a single unique note ID.
    Unique,
    /// Filter by note nullifiers (currently unused placeholder).
    Nullifiers,
    /// Only include notes that are unverified.
    Unverified,
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
