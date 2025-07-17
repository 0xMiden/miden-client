use miden_objects::account::AccountStorage as NativeAccountStorage;
use wasm_bindgen::prelude::*;

use crate::models::word::Word;

#[derive(Clone)]
#[wasm_bindgen]
pub struct AccountStorage(NativeAccountStorage);

#[wasm_bindgen]
impl AccountStorage {
    pub fn commitment(&self) -> Word {
        self.0.commitment().into()
    }

    #[wasm_bindgen(js_name = "getItem")]
    pub fn get_item(&self, index: u8) -> Option<Word> {
        self.0.get_item(index).ok().map(Into::into)
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
