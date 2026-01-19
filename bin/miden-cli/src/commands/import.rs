use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;

use miden_client::account::{AccountFile, AccountId};
use miden_client::auth::TransactionAuthenticator;
use miden_client::note::NoteFile;
use miden_client::utils::Deserializable;
use miden_client::{Client, ClientError};
use tracing::{info, warn};

use crate::commands::account::set_default_account_if_unset;
use crate::errors::CliError;
use crate::{FilesystemKeyStore, Parser};

#[derive(Debug, Parser, Clone)]
#[command(about = "Import notes or accounts")]
pub struct ImportCmd {
    /// Paths to the files that contains the account/note data.
    #[arg()]
    filenames: Vec<PathBuf>,
    /// Only relevant for accounts. If set, the account will be overwritten if it already exists.
    #[arg(short, long, default_value_t = false)]
    overwrite: bool,
}

impl ImportCmd {
    pub async fn execute<AUTH: TransactionAuthenticator + Sync + 'static>(
        &self,
        mut client: Client<AUTH>,
        keystore: FilesystemKeyStore,
    ) -> Result<(), CliError> {
        validate_paths(&self.filenames)?;
        for filename in &self.filenames {
            let note_file = read_note_file(filename.clone());

            if let Ok(note_file) = note_file {
                let note_id = client.import_notes(&[note_file]).await?[0];
                println!("Successfully imported note {}", note_id.to_hex());
            } else {
                info!(
                    "Attempting to import account data from {}...",
                    fs::canonicalize(filename)?.as_path().display()
                );
                let account_file = AccountFile::read(filename)?;
                let account_id =
                    import_account(&mut client, &keystore, account_file, self.overwrite).await?;

                println!("Successfully imported account {account_id}");

                if account_id.is_regular_account() {
                    set_default_account_if_unset(&mut client, account_id).await?;
                }
            }
        }
        Ok(())
    }
}

// IMPORT ACCOUNT
// ================================================================================================

/// Imports an account file to the client.
///
/// This implies:
///
/// - Reading all secret keys, and importing it to the CLI keystore
/// - Storing account ID -> public key commitment mapping on the client's store
/// - Adding the [account][`miden_client::account::Account`] to the client
async fn import_account<AUTH>(
    client: &mut Client<AUTH>,
    keystore: &FilesystemKeyStore,
    account_file: AccountFile,
    overwrite: bool,
) -> Result<AccountId, CliError> {
    let account_id = account_file.account.id();
    let AccountFile { account, auth_secret_keys } = account_file;

    for key in auth_secret_keys {
        let public_key = key.public_key();
        let public_key_commitment = public_key.to_commitment();
        keystore.add_key(&key).map_err(CliError::KeyStore)?;
        if let Err(err) =
            client.register_account_public_key_commitments(&account_id, &[public_key]).await
        {
            if let Err(rollback_err) = keystore.remove_key(public_key_commitment) {
                warn!(
                    ?rollback_err,
                    "Failed to rollback keystore entry after commitment add failure"
                );
            }
            return Err(err.into());
        }
    }

    client.add_account(&account, overwrite).await?;

    Ok(account_id)
}

// IMPORT NOTE
// ================================================================================================

fn read_note_file(filename: PathBuf) -> Result<NoteFile, CliError> {
    let mut contents = vec![];
    let mut _file = File::open(filename).and_then(|mut f| f.read_to_end(&mut contents))?;

    NoteFile::read_from_bytes(&contents)
        .map_err(|err| ClientError::DataDeserializationError(err).into())
}

// HELPERS
// ================================================================================================

/// Checks that all files exist, otherwise returns an error. It also ensures that all files have a
/// specific extension.
fn validate_paths(paths: &[PathBuf]) -> Result<(), CliError> {
    let invalid_path = paths.iter().find(|path| !path.exists());

    if let Some(path) = invalid_path {
        Err(CliError::Input(format!("The path `{}` does not exist", path.to_string_lossy())))
    } else {
        Ok(())
    }
}
