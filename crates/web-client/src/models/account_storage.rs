use miden_objects::account::AccountStorage as NativeAccountStorage;
use wasm_bindgen::prelude::*;

use crate::models::word::Word;

#[derive(Clone)]
#[wasm_bindgen]
pub struct AccountStorage(NativeAccountStorage);

#[wasm_bindgen]
impl AccountStorage {
    /// Returns a commitment to this storage.
    pub fn commitment(&self) -> Word {
        self.0.commitment().into()
    }

    /// Returns an item from the storage at the specified index.
    ///
    /// @remarks
    /// Errors:
    /// - If the index is out of bounds
    ///
    /// @param index - The slot index in storage.
    /// @returns The stored `Word`, or `undefined` if not found.
    #[wasm_bindgen(js_name = "getItem")]
    pub fn get_item(&self, index: u8) -> Option<Word> {
        self.0.get_item(index).ok().map(Into::into)
    }

    /// Returns a map item from a map located in storage at the specified index.
    ///
    /// @remarks
    /// Errors:
    /// - If the index is out of bounds
    /// - If the indexed storage slot is not a map
    ///
    /// @param index - The slot index in storage.
    /// @param key - The key used to look up the map item.
    /// @returns The stored `Word`, or `undefined` if not found.
    #[wasm_bindgen(js_name = "getMapItem")]
    pub fn get_map_item(&self, index: u8, key: Word) -> Option<Word> {
        self.0.get_map_item(index, key.into()).ok().map(Into::into)
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeAccountStorage> for AccountStorage {
    fn from(native_account_storage: NativeAccountStorage) -> Self {
        AccountStorage(native_account_storage)
    }
}

impl From<&NativeAccountStorage> for AccountStorage {
    fn from(native_account_storage: &NativeAccountStorage) -> Self {
        AccountStorage(native_account_storage.clone())
    }
}
