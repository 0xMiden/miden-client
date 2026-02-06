//! Domain structs representing serialized account query rows.

use miden_client::Word;
use miden_client::account::{StorageSlotName, StorageSlotType};
use miden_client::store::StoreError;
use rusqlite::Row;

use crate::account::query::parse::{parse_latest_storage_row, parse_slot_value};
use crate::column_value_as_u64;

pub(crate) struct LatestStorageSlotRow {
    pub name: String,
    pub value: String,
    pub slot_type: u8,
}

impl LatestStorageSlotRow {
    pub(crate) fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            name: row.get(0)?,
            value: row.get(1)?,
            slot_type: row.get(2)?,
        })
    }

    pub(crate) fn parse(self) -> Result<(StorageSlotName, (StorageSlotType, Word)), StoreError> {
        parse_latest_storage_row(self.name, self.value, self.slot_type)
    }
}

pub(crate) struct StorageSlotValueRow {
    pub value: String,
    pub slot_type: u8,
}

impl StorageSlotValueRow {
    pub(crate) fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            value: row.get(0)?,
            slot_type: row.get(1)?,
        })
    }

    pub(crate) fn parse(self) -> Result<(StorageSlotType, Word), StoreError> {
        parse_slot_value(self.value, self.slot_type)
    }
}

pub(crate) struct StorageSlotValueNonceRow {
    pub value: String,
    pub slot_type: u8,
    pub nonce: u64,
}

impl StorageSlotValueNonceRow {
    pub(crate) fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            value: row.get(0)?,
            slot_type: row.get(1)?,
            nonce: column_value_as_u64(row, 2)?,
        })
    }
}
