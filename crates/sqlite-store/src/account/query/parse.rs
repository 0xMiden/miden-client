//! Shared parsing helpers for account query modules.

use miden_client::Word;
use miden_client::account::{StorageSlotName, StorageSlotType};
use miden_client::store::StoreError;

fn parse_error(err: impl std::fmt::Display) -> StoreError {
    StoreError::ParsingError(err.to_string())
}

pub(crate) fn parse_slot_name(slot_name: String) -> Result<StorageSlotName, StoreError> {
    StorageSlotName::new(slot_name).map_err(parse_error)
}

pub(crate) fn parse_slot_type(slot_type: u8) -> Result<StorageSlotType, StoreError> {
    StorageSlotType::try_from(slot_type).map_err(parse_error)
}

pub(crate) fn parse_word(hex_word: String) -> Result<Word, StoreError> {
    Ok(Word::try_from(hex_word)?)
}

pub(crate) fn parse_slot_value(
    slot_value: String,
    slot_type: u8,
) -> Result<(StorageSlotType, Word), StoreError> {
    Ok((parse_slot_type(slot_type)?, parse_word(slot_value)?))
}

pub(crate) fn parse_latest_storage_row(
    slot_name: String,
    slot_value: String,
    slot_type: u8,
) -> Result<(StorageSlotName, (StorageSlotType, Word)), StoreError> {
    Ok((parse_slot_name(slot_name)?, parse_slot_value(slot_value, slot_type)?))
}
