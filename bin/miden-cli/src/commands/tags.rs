use miden_client::Client;
use miden_client::note::{NoteExecutionMode, NoteTag};
use tracing::info;

use crate::errors::CliError;
use crate::{Parser, create_dynamic_table};

#[derive(Default, Debug, Parser, Clone)]
#[command(about = "View and manage tags. Defaults to `list` command")]
pub struct TagsCmd {
    /// List all tags monitored by this client.
    #[arg(short, long, group = "action")]
    list: bool,

    /// Add a new tag to the list of tags monitored by this client.
    #[arg(short, long, group = "action", value_name = "tag")]
    add: Option<u32>,

    /// Removes a tag from the list of tags monitored by this client.
    #[arg(short, long, group = "action", value_name = "tag")]
    remove: Option<u32>,
}

impl TagsCmd {
    pub async fn execute<AUTH>(&self, client: Client<AUTH>) -> Result<(), CliError> {
        match self {
            TagsCmd { add: Some(tag), .. } => {
                add_tag(client, *tag).await?;
            },
            TagsCmd { remove: Some(tag), .. } => {
                remove_tag(client, *tag).await?;
            },
            _ => {
                list_tags(client).await?;
            },
        }
        Ok(())
    }
}

// HELPERS
// ================================================================================================
async fn list_tags<AUTH>(client: Client<AUTH>) -> Result<(), CliError> {
    let mut table = create_dynamic_table(&["Tag", "Source"]);

    let tags = client.get_note_tags().await?;

    for tag in tags {
        let source = match tag.source {
            miden_client::sync::NoteTagSource::Account(account_id) => {
                format!("Account({})", account_id.to_hex())
            },
            miden_client::sync::NoteTagSource::Note(note_id) => format!("Note({note_id})"),
            miden_client::sync::NoteTagSource::User => "User".to_string(),
        };

        table.add_row(vec![tag.tag.to_string(), source]);
    }

    println!("\n{table}");

    Ok(())
}

async fn add_tag<AUTH>(mut client: Client<AUTH>, tag: u32) -> Result<(), CliError> {
    let tag: NoteTag = tag.into();
    let execution_mode = match tag.execution_mode() {
        NoteExecutionMode::Local => "Local",
        NoteExecutionMode::Network => "Network",
    };
    info!(
        "adding tag - Single Target? {} - Execution mode: {}",
        tag.is_single_target(),
        execution_mode
    );
    client.add_note_tag(tag).await?;
    println!("Tag {tag} added");
    Ok(())
}

async fn remove_tag<AUTH>(mut client: Client<AUTH>, tag: u32) -> Result<(), CliError> {
    client.remove_note_tag(tag.into()).await?;
    println!("Tag {tag} removed");
    Ok(())
}
