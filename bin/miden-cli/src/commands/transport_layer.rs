use miden_client::address::Address;
use miden_client::auth::TransactionAuthenticator;
use miden_client::note::{Note, NoteId};
use miden_client::{Client, ClientError};

use crate::Parser;
use crate::errors::CliError;

#[derive(Debug, Parser, Clone)]
#[command(about = "Exchange privates notes using the Transport Layer")]
pub enum TransportLayerCmd {
    Send { note_id: String, address: String },
    Fetch,
}

impl TransportLayerCmd {
    pub async fn execute<AUTH: TransactionAuthenticator + Sync + 'static>(
        &self,
        mut client: Client<AUTH>,
    ) -> Result<(), CliError> {
        if !client.transport_layer().is_enabled() {
            return Err(CliError::Config(
                "Missing configuration".to_string().into(),
                "Please provide a [transport-layer] configuration to use the transport layer"
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

    client.transport_layer().send_note(note, &address).await?;

    Ok(())
}

// FETCH
// ================================================================================================

/// Retrieve notes for all tracked tags
///
/// Fetched notes are stored in the store.
async fn fetch<AUTH>(client: &mut Client<AUTH>) -> Result<(), CliError> {
    client.transport_layer().fetch_notes().await?;

    Ok(())
}
