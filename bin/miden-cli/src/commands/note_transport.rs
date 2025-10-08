use miden_client::address::Address;
use miden_client::auth::TransactionAuthenticator;
use miden_client::note::{Note, NoteId};
use miden_client::{Client, ClientError};

use crate::Parser;
use crate::errors::CliError;

#[derive(Debug, Parser, Clone)]
#[command(about = "Exchange privates notes using the Note Transport network")]
pub enum NoteTransportCmd {
    /// Send a private note through the Note Transport network.
    Send {
        /// Note ID of the sending note, as a hex string.
        note_id: String,
        /// Address of the recipient, as a Bech32 string.
        address: String,
    },
    /// Fetch notes from the Note Transport network. Fetched notes will be added to the store.
    Fetch,
}

impl NoteTransportCmd {
    pub async fn execute<AUTH: TransactionAuthenticator + Sync + 'static>(
        &self,
        mut client: Client<AUTH>,
    ) -> Result<(), CliError> {
        if !client.is_note_transport_enabled() {
            return Err(CliError::Config(
                "Missing configuration".to_string().into(),
                "Please provide a [note_transport] configuration to use the note transport network"
                    .to_string(),
            ));
        }

        match self {
            Self::Send { note_id, address } => send(&mut client, note_id, address).await,
            Self::Fetch => fetch(&mut client).await,
        }
    }
}

// SEND
// ================================================================================================

/// Send a (stored) note
async fn send<AUTH: TransactionAuthenticator + Sync>(
    client: &mut Client<AUTH>,
    note_id: &str,
    address: &str,
) -> Result<(), CliError> {
    let id = NoteId::try_from_hex(note_id).map_err(|e| CliError::Input(e.to_string()))?;
    let note_record = client
        .get_input_note(id)
        .await?
        .ok_or_else(|| CliError::Input(format!("note {note_id} not found")))?;
    let note: Note = note_record
        .try_into()
        .map_err(|e| CliError::Client(ClientError::NoteRecordConversionError(e)))?;
    let (_netid, address) =
        Address::from_bech32(address).map_err(|e| CliError::Input(e.to_string()))?;

    client.send_private_note(note, &address).await?;

    Ok(())
}

// FETCH
// ================================================================================================

/// Retrieve notes for all tracked tags
///
/// Fetched notes are stored in the store.
async fn fetch<AUTH>(client: &mut Client<AUTH>) -> Result<(), CliError> {
    client.fetch_private_notes().await?;

    Ok(())
}
