#![no_std]
#![feature(alloc_error_handler)]

use miden::{Asset, NoteIdx, output_note};

#[miden::component]
struct MyAccount;

#[miden::component]
impl MyAccount {
    /// Adds an asset to the account.
    ///
    /// This function adds the specified asset to the account's asset list.
    ///
    /// # Arguments
    /// * `asset` - The asset to be added to the account
    pub fn receive_asset(&mut self, asset: Asset) {
        self.add_asset(asset);
    }

    /// Moves an asset from the account to a note.
    ///
    /// This function removes the specified asset from the account and adds it to
    /// the note identified by the given index.
    ///
    /// # Arguments
    /// * `asset` - The asset to move from the account to the note
    /// * `note_idx` - The index of the note to receive the asset
    pub fn move_asset_to_note(&mut self, asset: Asset, note_idx: NoteIdx) {
        let asset = self.remove_asset(asset);
        output_note::add_asset(asset, note_idx);
    }
}
