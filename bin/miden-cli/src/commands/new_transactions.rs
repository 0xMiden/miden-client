use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::{fs, io};

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use clap::{Parser, ValueEnum};
use miden_client::account::AccountId;
use miden_client::asset::{FungibleAsset, NonFungibleDeltaAction};
use miden_client::auth::TransactionAuthenticator;
use miden_client::note::{
    BlockNumber,
    Note,
    NoteFile,
    NoteId,
    NoteType as MidenNoteType,
    build_swap_tag,
    get_input_note_with_id_prefix,
};
use miden_client::store::NoteRecordError;
use miden_client::transaction::{
    ExecutedTransaction,
    InputNote,
    OutputNote,
    PaymentNoteDescription,
    SwapTransactionData,
    TransactionId,
    TransactionRequest,
    TransactionRequestBuilder,
};
use miden_client::{Client, Deserializable, RemoteTransactionProver};
use rand::Rng;
use reqwest::{Client as HttpClient, Url};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::task;
use tokio::time::sleep;
use tracing::{debug, info};
use {hex, serde_json};

use crate::create_dynamic_table;
use crate::errors::CliError;
use crate::utils::{
    SHARED_TOKEN_DOCUMENTATION,
    get_input_acc_id_by_prefix_or_default,
    load_config_file,
    load_faucet_details_map,
    parse_account_id,
};

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum NoteType {
    Public,
    Private,
}

impl From<&NoteType> for MidenNoteType {
    fn from(note_type: &NoteType) -> Self {
        match note_type {
            NoteType::Public => MidenNoteType::Public,
            NoteType::Private => MidenNoteType::Private,
        }
    }
}

/// Mint tokens by requesting them from the faucet API (with PoW).
#[derive(Debug, Parser, Clone)]
pub struct MintCmd {
    /// Amount to be minted.
    #[arg(short = 'a', long = "amount", help = "Amount to be minted from the faucet.")]
    amount: u64,

    /// Target account ID or its hex prefix for the minted tokens. If none is provided, the default
    /// account's ID is used instead.
    #[arg(short = 't', long = "target")]
    target_account_id: Option<String>,

    /// Optional faucet API key.
    #[arg(long = "api-key")]
    api_key: Option<String>,

    /// If set, also write the downloaded note file to this path (in addition to importing it).
    #[arg(long = "note-path")]
    note_output_path: Option<PathBuf>,
}

impl MintCmd {
    pub async fn execute<AUTH: TransactionAuthenticator + Sync + 'static>(
        &self,
        mut client: Client<AUTH>,
    ) -> Result<(), CliError> {
        if self.amount == 0 {
            return Err(CliError::Input("Amount must be greater than zero".to_string()));
        }

        let target_account_id =
            get_input_acc_id_by_prefix_or_default(&client, self.target_account_id.clone()).await?;

        let (cli_config, _) = load_config_file()?;
        let faucet_config = cli_config.faucet;

        let faucet_url = Url::parse(&faucet_config.endpoint).map_err(|err| {
            CliError::Faucet(format!("Invalid faucet URL `{}`: {err}", faucet_config.endpoint))
        })?;

        let http_client = HttpClient::builder()
            .timeout(Duration::from_millis(faucet_config.timeout_ms))
            .build()
            .map_err(|err| CliError::Faucet(format!("Failed to build HTTP client: {err}")))?;

        println!("Requesting tokens from faucet...");
        let (pow_challenge, pow_target) = request_pow(
            &http_client,
            &faucet_url,
            &target_account_id,
            self.amount,
            self.api_key.as_deref(),
        )
        .await?;

        let nonce = solve_challenge(pow_challenge.clone(), pow_target).await?;

        let note_id_str = request_tokens(
            &http_client,
            &faucet_url,
            &pow_challenge,
            nonce,
            &target_account_id,
            self.amount,
            NoteType::Private,
            self.api_key.as_deref(),
        )
        .await?;

        println!("Faucet accepted mint request");

        let note_bytes = download_note(&http_client, &faucet_url, &note_id_str).await?;

        if let Some(path) = &self.note_output_path {
            fs::write(path, &note_bytes).map_err(|err| {
                CliError::Import(format!("Failed to write note to {}: {err}", path.display()))
            })?;
        }

        let note_file = NoteFile::read_from_bytes(&note_bytes)
            .map_err(|err| CliError::Import(format!("Failed to decode faucet note: {err}")))?;
        let imported_note_id = client.import_note(note_file.clone()).await?;

        // Build an unauthenticated consume transaction from the imported note record.
        let input_note = build_input_note_for_consumption(
            &mut client,
            &http_client,
            &faucet_url,
            imported_note_id,
            note_file,
        )
        .await?;

        let transaction_request = TransactionRequestBuilder::new()
            .unauthenticated_input_notes(vec![(input_note_into_note(input_note), None)])
            .build()
            .map_err(|err| {
                CliError::Transaction(
                    err.into(),
                    "Failed to build consume notes transaction".to_string(),
                )
            })?;

        let transaction_id =
            execute_transaction(&mut client, target_account_id, transaction_request, true, false)
                .await?;
        println!(
            "View the mint transaction on Midenscan: https://midenscan.com/transaction/{}",
            transaction_id
        );

        Ok(())
    }
}

// FAUCET HELPERS
// ================================================================================================

#[derive(Debug, Deserialize)]
struct PowResponse {
    challenge: String,
    target: u64,
}

// Request a PoW from the faucet API.
async fn request_pow(
    http_client: &HttpClient,
    base_url: &Url,
    account_id: &AccountId,
    amount: u64,
    api_key: Option<&str>,
) -> Result<(String, u64), CliError> {
    let pow_url = base_url.join("pow").map_err(|err| {
        CliError::Faucet(format!("Failed to construct PoW endpoint from {}: {err}", base_url))
    })?;

    let mut request = http_client
        .get(pow_url)
        .query(&[("account_id", account_id.to_hex()), ("amount", amount.to_string())]);

    if let Some(key) = api_key {
        request = request.query(&[("api_key", key)]);
    }

    let response = request
        .send()
        .await
        .map_err(|err| CliError::Faucet(format!("PoW request failed: {err}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::Faucet(format!("Faucet PoW request failed ({}): {}", status, body)));
    }

    let body = response.text().await.unwrap_or_default();
    let response: PowResponse = serde_json::from_str(&body)
        .map_err(|err| CliError::Faucet(format!("Failed to parse PoW response: {err}")))?;

    Ok((response.challenge, response.target))
}

#[derive(Debug, Deserialize)]
struct MintResponse {
    note_id: String,
}

// Request tokens from the faucet API.
async fn request_tokens(
    http_client: &HttpClient,
    base_url: &Url,
    challenge: &str,
    nonce: u64,
    account_id: &AccountId,
    amount: u64,
    note_type: NoteType,
    api_key: Option<&str>,
) -> Result<String, CliError> {
    let url = base_url.join("get_tokens").map_err(|err| {
        CliError::Faucet(format!(
            "Failed to construct get_tokens endpoint from {}: {err}",
            base_url
        ))
    })?;

    let mut request = http_client.get(url).query(&[
        ("account_id", account_id.to_hex()),
        ("asset_amount", amount.to_string()),
        ("is_private_note", (note_type == NoteType::Private).to_string()),
        ("challenge", challenge.to_string()),
        ("nonce", nonce.to_string()),
    ]);

    if let Some(key) = api_key {
        request = request.query(&[("api_key", key)]);
    }

    let response = request
        .send()
        .await
        .map_err(|err| CliError::Faucet(format!("get_tokens request failed: {err}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::Faucet(format!(
            "Faucet get_tokens request failed ({}): {}",
            status, body
        )));
    }

    let response: MintResponse = response
        .json()
        .await
        .map_err(|err| CliError::Faucet(format!("Failed to parse get_tokens response: {err}")))?;

    Ok(response.note_id)
}

// Response from the faucet API for a private note.
#[derive(Debug, Deserialize)]
struct NoteResponse {
    data_base64: String,
}

// Download a private note from the faucet API.
async fn download_note(
    http_client: &HttpClient,
    base_url: &Url,
    note_id: &str,
) -> Result<Vec<u8>, CliError> {
    let url = base_url.join("get_note").map_err(|err| {
        CliError::Faucet(format!("Failed to construct get_note endpoint from {}: {err}", base_url))
    })?;

    let response = http_client
        .get(url)
        .query(&[("note_id", note_id.to_string())])
        .send()
        .await
        .map_err(|err| CliError::Faucet(format!("Failed to download note: {err}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(CliError::Faucet(format!(
            "Faucet get_note request failed ({}): {}",
            status, body
        )));
    }

    let response: NoteResponse = response
        .json()
        .await
        .map_err(|err| CliError::Faucet(format!("Failed to parse get_note response: {err}")))?;

    BASE64_STANDARD
        .decode(response.data_base64)
        .map_err(|err| CliError::Import(format!("Failed to decode note payload: {err}")))
}

// Solve a PoW challenge for the given challenge and target from the faucet API.
async fn solve_challenge(challenge_hex: String, target: u64) -> Result<u64, CliError> {
    if target == 0 {
        return Err(CliError::Faucet("Received PoW target of 0 from faucet".to_string()));
    }

    let challenge_bytes = hex::decode(challenge_hex).map_err(|err| {
        CliError::Faucet(format!("Invalid challenge bytes returned by faucet: {err}"))
    })?;

    task::spawn_blocking(move || {
        let mut rng = rand::rng();

        loop {
            let nonce: u64 = rng.random();

            let mut hasher = Sha256::new();
            hasher.update(&challenge_bytes);
            hasher.update(nonce.to_be_bytes());
            let hash = hasher.finalize();
            let digest =
                u64::from_be_bytes(hash[..8].try_into().expect("hash should be 32 bytes long"));

            if digest < target {
                return Ok(nonce);
            }
        }
    })
    .await
    .map_err(|err| CliError::Faucet(format!("PoW solving task failed: {err}")))?
}

fn input_note_into_note(input_note: InputNote) -> Note {
    match input_note {
        InputNote::Authenticated { note, .. } => note,
        InputNote::Unauthenticated { note } => note,
    }
}

// Build an unauthenticated input note from a note file.
async fn build_input_note_for_consumption<AUTH: TransactionAuthenticator + Sync + 'static>(
    client: &mut Client<AUTH>,
    http_client: &HttpClient,
    faucet_url: &Url,
    note_id: NoteId,
    note_file: NoteFile,
) -> Result<InputNote, CliError> {
    const NOTE_READY_TIMEOUT_SECS: u64 = 180;
    const RETRY_DELAY_SECS: u64 = 2;

    // Best case: faucet already returns a proof, so we can consume immediately.
    if let NoteFile::NoteWithProof(note, proof) = note_file {
        return Ok(InputNote::authenticated(note, proof));
    }

    let start = std::time::Instant::now();
    loop {
        // Try to build from the stored record first.
        if let Ok(note_record) =
            get_input_note_with_id_prefix(client, &note_id.to_hex()).await.map_err(|err| {
                CliError::Transaction(
                    err.into(),
                    "Failed to locate imported faucet note in local store".to_string(),
                )
            })
        {
            match note_record.try_into() {
                Ok(input) => return Ok(input),
                Err(NoteRecordError::ConversionError(_)) => {
                    // Missing metadata/proof; retry below after waiting for commitment/proof
                    // export.
                },
                Err(err) => {
                    return Err(CliError::Transaction(
                        err.into(),
                        "Failed to prepare faucet note for consumption".to_string(),
                    ));
                },
            }
        }

        // Re-fetch periodically; once committed the faucet exports NoteWithProof.
        if let Ok(bytes) = download_note(http_client, faucet_url, &note_id.to_hex()).await {
            if let Ok(fresh_note_file) = NoteFile::read_from_bytes(&bytes) {
                let _ = client.import_note(fresh_note_file).await;
            }
        }

        // Sync and wait before retrying.
        client.sync_state().await?;
        debug!("Waiting for faucet note {} to become consumable", note_id);

        if start.elapsed().as_secs() >= NOTE_READY_TIMEOUT_SECS {
            return Err(CliError::Transaction(
                "Imported faucet note is not yet consumable; timed out waiting for metadata/proof"
                    .into(),
                "Faucet note not yet consumable".to_string(),
            ));
        }

        sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
    }
}

/// Create a pay-to-id transaction.
#[derive(Debug, Parser, Clone)]
pub struct SendCmd {
    /// Sender account ID or its hex prefix. If none is provided, the default account's ID is used
    /// instead.
    #[arg(short = 's', long = "sender")]
    sender_account_id: Option<String>,
    /// Target account ID or its hex prefix.
    #[arg(short = 't', long = "target")]
    target_account_id: String,

    /// Asset to be sent.
    #[arg(short, long, help=format!("Asset to be sent.\n{SHARED_TOKEN_DOCUMENTATION}"))]
    asset: String,

    #[arg(short, long, value_enum)]
    note_type: NoteType,
    /// Flag to submit the executed transaction without asking for confirmation
    #[arg(long, default_value_t = false)]
    force: bool,
    /// Set the recall height for the transaction. If the note wasn't consumed by this height, the
    /// sender may consume it back.
    ///
    /// Setting this flag turns the transaction from a `PayToId` to a `PayToIdWithRecall`.
    #[arg(short, long)]
    recall_height: Option<u32>,

    /// Set the timelock height for the transaction. The note will not be consumable until this
    /// height is reached.
    #[arg(short = 'i', long)]
    timelock_height: Option<u32>,

    /// Flag to delegate proving to the remote prover specified in the config file
    #[arg(long, default_value_t = false)]
    delegate_proving: bool,
}

impl SendCmd {
    pub async fn execute<AUTH: TransactionAuthenticator + Sync + 'static>(
        &self,
        mut client: Client<AUTH>,
    ) -> Result<(), CliError> {
        let force = self.force;

        let faucet_details_map = load_faucet_details_map()?;

        let fungible_asset = faucet_details_map.parse_fungible_asset(&client, &self.asset).await?;

        // try to use either the provided argument or the default account
        let sender_account_id =
            get_input_acc_id_by_prefix_or_default(&client, self.sender_account_id.clone()).await?;
        let target_account_id = parse_account_id(&client, self.target_account_id.as_str()).await?;

        let mut payment_description = PaymentNoteDescription::new(
            vec![fungible_asset.into()],
            sender_account_id,
            target_account_id,
        );

        if let Some(recall_height) = self.recall_height {
            payment_description =
                payment_description.with_reclaim_height(BlockNumber::from(recall_height));
        }

        if let Some(timelock_height) = self.timelock_height {
            payment_description =
                payment_description.with_timelock_height(BlockNumber::from(timelock_height));
        }

        let transaction_request = TransactionRequestBuilder::new()
            .build_pay_to_id(payment_description, (&self.note_type).into(), client.rng())
            .map_err(|err| {
                CliError::Transaction(err.into(), "Failed to build payment transaction".to_string())
            })?;

        execute_transaction(
            &mut client,
            sender_account_id,
            transaction_request,
            force,
            self.delegate_proving,
        )
        .await
        .map(|_| ())
    }
}

/// Create a swap transaction.
#[derive(Debug, Parser, Clone)]
pub struct SwapCmd {
    /// Sender account ID or its hex prefix. If none is provided, the default account's ID is used
    /// instead.
    #[arg(short = 's', long = "source")]
    sender_account_id: Option<String>,

    /// Asset offered.
    #[arg(long = "offered-asset", help=format!("Asset offered.\n{SHARED_TOKEN_DOCUMENTATION}"))]
    offered_asset: String,

    /// Asset requested.
    #[arg(short, long, help=format!("Asset requested.\n{SHARED_TOKEN_DOCUMENTATION}"))]
    requested_asset: String,

    /// Visibility of the swap note to be created.
    #[arg(short, long, value_enum)]
    note_type: NoteType,

    /// Visibility of the payback note.
    #[arg(short, long, value_enum)]
    payback_note_type: NoteType,

    /// Flag to submit the executed transaction without asking for confirmation.
    #[arg(long, default_value_t = false)]
    force: bool,

    /// Flag to delegate proving to the remote prover specified in the config file.
    #[arg(long, default_value_t = false)]
    delegate_proving: bool,
}

impl SwapCmd {
    pub async fn execute<AUTH: TransactionAuthenticator + Sync + 'static>(
        &self,
        mut client: Client<AUTH>,
    ) -> Result<(), CliError> {
        let force = self.force;

        let faucet_details_map = load_faucet_details_map()?;

        let offered_fungible_asset =
            faucet_details_map.parse_fungible_asset(&client, &self.offered_asset).await?;
        let requested_fungible_asset =
            faucet_details_map.parse_fungible_asset(&client, &self.requested_asset).await?;

        // try to use either the provided argument or the default account
        let sender_account_id =
            get_input_acc_id_by_prefix_or_default(&client, self.sender_account_id.clone()).await?;

        let swap_transaction = SwapTransactionData::new(
            sender_account_id,
            offered_fungible_asset.into(),
            requested_fungible_asset.into(),
        );

        let transaction_request = TransactionRequestBuilder::new()
            .build_swap(
                &swap_transaction,
                (&self.note_type).into(),
                (&self.payback_note_type).into(),
                client.rng(),
            )
            .map_err(|err| {
                CliError::Transaction(err.into(), "Failed to build swap transaction".to_string())
            })?;

        execute_transaction(
            &mut client,
            sender_account_id,
            transaction_request,
            force,
            self.delegate_proving,
        )
        .await
        .map(|_| ())?;

        let payback_note_tag: u32 = build_swap_tag(
            (&self.note_type).into(),
            &swap_transaction.offered_asset(),
            &swap_transaction.requested_asset(),
        )
        .map_err(|err| CliError::Transaction(err.into(), "Failed to build swap tag".to_string()))?
        .into();
        println!(
            "To receive updates about the payback Swap Note run `miden tags add {payback_note_tag}`",
        );

        Ok(())
    }
}

/// Consume with the account corresponding to `account_id` all of the notes from `list_of_notes`.
/// If no account ID is provided, the default one is used. If no notes are provided, any notes
/// that are identified to be owned by the account ID are consumed.
#[derive(Debug, Parser, Clone)]
pub struct ConsumeNotesCmd {
    /// The account ID to be used to consume the note or its hex prefix. If none is provided, the
    /// default account's ID is used instead.
    #[arg(short = 'a', long = "account")]
    account_id: Option<String>,
    /// A list of note IDs or the hex prefixes of their corresponding IDs.
    list_of_notes: Vec<String>,
    /// Flag to submit the executed transaction without asking for confirmation.
    #[arg(short, long, default_value_t = false)]
    force: bool,

    /// Flag to delegate proving to the remote prover specified in the config file.
    #[arg(long, default_value_t = false)]
    delegate_proving: bool,
}

impl ConsumeNotesCmd {
    pub async fn execute<AUTH: TransactionAuthenticator + Sync + 'static>(
        &self,
        mut client: Client<AUTH>,
    ) -> Result<(), CliError> {
        let force = self.force;

        let mut authenticated_notes = Vec::new();
        let mut unauthenticated_notes = Vec::new();

        for note_id in &self.list_of_notes {
            let note_record = get_input_note_with_id_prefix(&client, note_id)
                .await
                .map_err(|_| CliError::Input(format!("Input note ID {note_id} is neither a valid Note ID nor a prefix of a known Note ID")))?;

            if note_record.is_authenticated() {
                authenticated_notes.push(note_record.id());
            } else {
                unauthenticated_notes.push((
                    note_record.try_into().map_err(|err: NoteRecordError| {
                        CliError::Transaction(
                            err.into(),
                            "Failed to convert note record".to_string(),
                        )
                    })?,
                    None,
                ));
            }
        }

        let account_id =
            get_input_acc_id_by_prefix_or_default(&client, self.account_id.clone()).await?;

        if authenticated_notes.is_empty() {
            info!("No input note IDs provided, getting all notes consumable by {}", account_id);
            let consumable_notes = client.get_consumable_notes(Some(account_id)).await?;

            authenticated_notes.extend(consumable_notes.iter().map(|(note, _)| note.id()));
        }

        if authenticated_notes.is_empty() && unauthenticated_notes.is_empty() {
            return Err(CliError::Transaction(
                "No input notes were provided and the store does not contain any notes consumable by {account_id}".into(),
                "Input notes check failed".to_string(),
            ));
        }

        let transaction_request = TransactionRequestBuilder::new()
            .authenticated_input_notes(authenticated_notes.into_iter().map(|id| (id, None)))
            .unauthenticated_input_notes(unauthenticated_notes)
            .build()
            .map_err(|err| {
                CliError::Transaction(
                    err.into(),
                    "Failed to build consume notes transaction".to_string(),
                )
            })?;

        execute_transaction(
            &mut client,
            account_id,
            transaction_request,
            force,
            self.delegate_proving,
        )
        .await
        .map(|_| ())
    }
}

// EXECUTE TRANSACTION
// ================================================================================================

async fn execute_transaction<AUTH: TransactionAuthenticator + Sync + 'static>(
    client: &mut Client<AUTH>,
    account_id: AccountId,
    transaction_request: TransactionRequest,
    force: bool,
    delegated_proving: bool,
) -> Result<TransactionId, CliError> {
    println!("Executing transaction...");
    let transaction_result = client.execute_transaction(account_id, transaction_request).await?;

    let executed_transaction = transaction_result.executed_transaction().clone();

    // Show delta and ask for confirmation
    print_transaction_details(&executed_transaction)?;
    if !force {
        println!(
            "\nContinue with proving and submission? Changes will be irreversible once the proof is finalized on the network (y/N)"
        );
        let mut proceed_str: String = String::new();
        io::stdin().read_line(&mut proceed_str).expect("Should read line");

        if proceed_str.trim().to_lowercase() != "y" {
            println!("Transaction was cancelled.");
            return Err(CliError::Transaction(
                std::io::Error::new(std::io::ErrorKind::Interrupted, "transaction cancelled")
                    .into(),
                "Transaction was cancelled by user".to_string(),
            ));
        }
    }

    let transaction_id = executed_transaction.id();
    let output_notes = executed_transaction
        .output_notes()
        .iter()
        .map(OutputNote::id)
        .collect::<Vec<_>>();

    println!("Proving transaction...");

    let prover = if delegated_proving {
        let (cli_config, _) = load_config_file()?;
        let remote_prover_endpoint =
            cli_config.remote_prover_endpoint.as_ref().ok_or(CliError::Config(
                "Remote prover endpoint".to_string().into(),
                "remote prover endpoint is not set in the configuration file".to_string(),
            ))?;

        Arc::new(RemoteTransactionProver::new(remote_prover_endpoint.to_string()))
    } else {
        client.prover()
    };

    let proven_transaction = client.prove_transaction_with(&transaction_result, prover).await?;

    println!("Submitting transaction to node...");

    let submission_height = client
        .submit_proven_transaction(proven_transaction, &transaction_result)
        .await?;
    println!("Applying transaction to store...");
    client.apply_transaction(&transaction_result, submission_height).await?;

    println!("Successfully created transaction.");
    println!("Transaction ID: {transaction_id}");

    if output_notes.is_empty() {
        println!("The transaction did not generate any output notes.");
    } else {
        println!("Output notes:");
        for note_id in &output_notes {
            println!("\t- {note_id}");
        }
    }

    Ok(transaction_id)
}

fn print_transaction_details(executed_tx: &ExecutedTransaction) -> Result<(), CliError> {
    println!("The transaction will have the following effects:\n");

    // INPUT NOTES
    let input_note_ids = executed_tx.input_notes().iter().map(InputNote::id).collect::<Vec<_>>();
    if input_note_ids.is_empty() {
        println!("No notes will be consumed.");
    } else {
        println!("The following notes will be consumed:");
        for input_note_id in input_note_ids {
            println!("\t- {}", input_note_id.to_hex());
        }
    }
    println!();

    // OUTPUT NOTES
    let output_note_count = executed_tx.output_notes().iter().count();
    if output_note_count == 0 {
        println!("No notes will be created as a result of this transaction.");
    } else {
        println!("{output_note_count} notes will be created as a result of this transaction.");
    }
    println!();

    // ACCOUNT CHANGES
    println!("The account with ID {} will be modified as follows:", executed_tx.account_id());

    let account_delta = executed_tx.account_delta();

    let has_storage_changes = !account_delta.storage().is_empty();
    if has_storage_changes {
        let mut table = create_dynamic_table(&["Storage Slot", "Effect"]);

        for (updated_item_slot, new_value) in account_delta.storage().values() {
            table.add_row(vec![
                updated_item_slot.to_string(),
                format!("Updated ({})", new_value.to_hex()),
            ]);
        }

        println!("Storage changes:");
        println!("{table}");
    } else {
        println!("Account Storage will not be changed.");
    }

    if account_delta.vault().is_empty() {
        println!("Account Vault will not be changed.");
    } else {
        let faucet_details_map = load_faucet_details_map()?;
        let mut table = create_dynamic_table(&["Asset Type", "Faucet ID", "Amount"]);

        for (faucet_id, amount) in account_delta.vault().fungible().iter() {
            let asset =
                FungibleAsset::new(*faucet_id, amount.unsigned_abs()).map_err(CliError::Asset)?;
            let (faucet_fmt, amount_fmt) = faucet_details_map.format_fungible_asset(&asset)?;

            if amount.is_positive() {
                table.add_row(vec!["Fungible Asset", &faucet_fmt, &format!("+{amount_fmt}")]);
            } else {
                table.add_row(vec!["Fungible Asset", &faucet_fmt, &format!("-{amount_fmt}")]);
            }
        }

        for (asset, action) in account_delta.vault().non_fungible().iter() {
            match action {
                NonFungibleDeltaAction::Add => {
                    table.add_row(vec![
                        "Non Fungible Asset",
                        &asset.faucet_id_prefix().to_hex(),
                        "1",
                    ]);
                },
                NonFungibleDeltaAction::Remove => {
                    table.add_row(vec![
                        "Non Fungible Asset",
                        &asset.faucet_id_prefix().to_hex(),
                        "-1",
                    ]);
                },
            }
        }

        println!("Vault changes:");
        println!("{table}");
    }

    println!("Nonce incremented by: {}.", account_delta.nonce_delta());

    Ok(())
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use super::*;

    #[tokio::test]
    async fn solve_challenge_finds_valid_nonce() {
        let challenge = "00".repeat(120);
        let target = u64::MAX;

        let nonce = solve_challenge(challenge.clone(), target)
            .await
            .expect("should solve challenge");

        let mut hasher = Sha256::new();
        hasher.update(vec![0u8; 120]);
        hasher.update(nonce.to_be_bytes());
        let digest = u64::from_be_bytes(hasher.finalize()[..8].try_into().unwrap());
        assert!(digest < target, "nonce should satisfy target");
    }
}
