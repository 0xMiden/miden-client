// NOTE FILTER (OUTPUT NOTES)
// ================================================================================================

use std::rc::Rc;

use miden_client::store::{InputNoteState, NoteFilter, OutputNoteState};
use rusqlite::types::Value;

type NoteQueryParams = Vec<Rc<Vec<Value>>>;

/// Returns the output notes query for a specific `NoteFilter`
pub(super) fn note_filter_to_query_output_notes(filter: &NoteFilter) -> (String, NoteQueryParams) {
    let base = "SELECT
                    note.recipient_digest,
                    note.assets,
                    note.metadata,
                    note.expected_height,
                    note.state
                    from output_notes AS note";

    let (condition, params) = note_filter_output_notes_condition(filter);
    let query = format!("{base} WHERE {condition}");

    (query, params)
}

/// Returns the WHERE clause  for a specific `NoteFilter`.
pub(super) fn note_filter_output_notes_condition(filter: &NoteFilter) -> (String, NoteQueryParams) {
    let mut params = Vec::new();
    let condition = match filter {
        NoteFilter::All => "1 = 1".to_string(),
        NoteFilter::Committed => {
            format!(
                "state_discriminant in ({}, {})",
                OutputNoteState::STATE_COMMITTED_PARTIAL,
                OutputNoteState::STATE_COMMITTED_FULL
            )
        },
        NoteFilter::Consumed => {
            format!("state_discriminant = {}", OutputNoteState::STATE_CONSUMED)
        },
        NoteFilter::Expected => {
            format!(
                "state_discriminant in ({}, {})",
                OutputNoteState::STATE_EXPECTED_PARTIAL,
                OutputNoteState::STATE_EXPECTED_FULL
            )
        },
        NoteFilter::Processing | NoteFilter::Unverified => "1 = 0".to_string(),
        NoteFilter::Unique(note_id) => {
            let note_ids_list = vec![Value::Text(note_id.as_word().to_string())];
            params.push(Rc::new(note_ids_list));
            "note.note_id IN rarray(?)".to_string()
        },
        NoteFilter::List(note_ids) => {
            let note_ids_list = note_ids
                .iter()
                .map(|note_id| Value::Text(note_id.as_word().to_string()))
                .collect::<Vec<Value>>();

            params.push(Rc::new(note_ids_list));
            "note.note_id IN rarray(?)".to_string()
        },
        NoteFilter::Nullifiers(nullifiers) => {
            let nullifiers_list = nullifiers
                .iter()
                .map(|nullifier| Value::Text(nullifier.to_string()))
                .collect::<Vec<Value>>();

            params.push(Rc::new(nullifiers_list));
            "note.nullifier IN rarray(?)".to_string()
        },
        NoteFilter::Unspent => {
            format!(
                "state_discriminant in ({}, {}, {}, {})",
                OutputNoteState::STATE_EXPECTED_PARTIAL,
                OutputNoteState::STATE_EXPECTED_FULL,
                OutputNoteState::STATE_COMMITTED_PARTIAL,
                OutputNoteState::STATE_COMMITTED_FULL,
            )
        },
    };

    (condition, params)
}

// NOTE FILTER (INPUT NOTES)
// ================================================================================================

pub(super) fn note_filter_to_query_input_notes(filter: &NoteFilter) -> (String, NoteQueryParams) {
    let base = "SELECT
                note.assets,
                note.serial_number,
                note.inputs,
                script.serialized_note_script,
                note.state,
                note.created_at
                from input_notes AS note
                LEFT OUTER JOIN notes_scripts AS script
                    ON note.script_root = script.script_root";

    let (condition, params) = note_filter_input_notes_condition(filter);
    let query = format!("{base} WHERE {condition}");

    (query, params)
}

/// Returns the WHERE clause for the input [`NoteFilter`]
pub(super) fn note_filter_input_notes_condition(filter: &NoteFilter) -> (String, NoteQueryParams) {
    let mut params = Vec::new();
    let condition = match filter {
        NoteFilter::All => "(1 = 1)".to_string(),
        NoteFilter::Committed => {
            format!("(state_discriminant = {})", InputNoteState::STATE_COMMITTED)
        },
        NoteFilter::Consumed => {
            format!(
                "(state_discriminant in ({}, {}, {}))",
                InputNoteState::STATE_CONSUMED_AUTHENTICATED_LOCAL,
                InputNoteState::STATE_CONSUMED_UNAUTHENTICATED_LOCAL,
                InputNoteState::STATE_CONSUMED_EXTERNAL
            )
        },
        NoteFilter::Expected => {
            format!("(state_discriminant = {})", InputNoteState::STATE_EXPECTED)
        },
        NoteFilter::Processing => {
            format!(
                "(state_discriminant in ({}, {}))",
                InputNoteState::STATE_PROCESSING_AUTHENTICATED,
                InputNoteState::STATE_PROCESSING_UNAUTHENTICATED
            )
        },
        NoteFilter::Unique(note_id) => {
            let note_ids_list = vec![Value::Text(note_id.as_word().to_string())];
            params.push(Rc::new(note_ids_list));
            "(note.note_id IN rarray(?))".to_string()
        },
        NoteFilter::List(note_ids) => {
            let note_ids_list = note_ids
                .iter()
                .map(|note_id| Value::Text(note_id.as_word().to_string()))
                .collect::<Vec<Value>>();

            params.push(Rc::new(note_ids_list));
            "(note.note_id IN rarray(?))".to_string()
        },
        NoteFilter::Nullifiers(nullifiers) => {
            let nullifiers_list = nullifiers
                .iter()
                .map(|nullifier| Value::Text(nullifier.to_string()))
                .collect::<Vec<Value>>();

            params.push(Rc::new(nullifiers_list));
            "(note.nullifier IN rarray(?))".to_string()
        },
        NoteFilter::Unverified => {
            format!("(state_discriminant = {})", InputNoteState::STATE_UNVERIFIED)
        },
        NoteFilter::Unspent => {
            format!(
                "(state_discriminant in ({}, {}, {}, {}, {}))",
                InputNoteState::STATE_EXPECTED,
                InputNoteState::STATE_PROCESSING_AUTHENTICATED,
                InputNoteState::STATE_PROCESSING_UNAUTHENTICATED,
                InputNoteState::STATE_UNVERIFIED,
                InputNoteState::STATE_COMMITTED
            )
        },
    };

    (condition, params)
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unspent_filter_includes_all_non_consumed_output_states() {
        let (condition, _params) = note_filter_output_notes_condition(&NoteFilter::Unspent);

        // Verify all non-consumed output note states are included
        assert!(condition.contains(&OutputNoteState::STATE_EXPECTED_PARTIAL.to_string()));
        assert!(condition.contains(&OutputNoteState::STATE_EXPECTED_FULL.to_string()));
        assert!(condition.contains(&OutputNoteState::STATE_COMMITTED_PARTIAL.to_string()));
        assert!(condition.contains(&OutputNoteState::STATE_COMMITTED_FULL.to_string()));

        // Verify consumed state is NOT included
        assert!(!condition.contains(&OutputNoteState::STATE_CONSUMED.to_string()));
    }
}
