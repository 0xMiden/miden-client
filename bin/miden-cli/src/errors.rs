use std::error::Error;

use miden_client::{ClientError, keystore::KeyStoreError, rpc::RpcError};
use miden_lib::utils::ScriptBuilderError;
use miden_objects::{AccountError, AccountIdError, AssetError, NetworkIdError};
use miette::Diagnostic;
use thiserror::Error;

use crate::CLIENT_BINARY_NAME;

type SourceError = Box<dyn Error + Send + Sync>;

#[derive(Debug, Diagnostic, Error)]
pub enum CliError {
    #[error("account error: {1}")]
    #[diagnostic(code(cli::account_error))]
    Account(#[source] AccountError, String),
    #[error("account component error: {1}")]
    #[diagnostic(code(cli::account_error))]
    AccountComponentError(#[source] SourceError, String),
    #[error("account id error: {1}")]
    #[diagnostic(code(cli::accountid_error), help("Check the account ID format."))]
    AccountId(#[source] AccountIdError, String),
    #[error("asset error")]
    #[diagnostic(code(cli::asset_error))]
    Asset(#[source] AssetError),
    #[error("{}", format_client_error(.0))]
    #[diagnostic(code(cli::client_error))]
    Client(#[from] ClientError),
    #[error("config error: {1}")]
    #[diagnostic(
        code(cli::config_error),
        help(
            "Check if the configuration file exists and is well-formed. If it does not exist, run `{CLIENT_BINARY_NAME} init` command to create it."
        )
    )]
    Config(#[source] SourceError, String),
    #[error("execute program error: {1}")]
    #[diagnostic(code(cli::execute_program_error))]
    Exec(#[source] SourceError, String),
    #[error("export error: {0}")]
    #[diagnostic(code(cli::export_error), help("Check the ID."))]
    Export(String),
    #[error("faucet error: {0}")]
    #[diagnostic(code(cli::faucet_error))]
    Faucet(String),
    #[error("import error: {0}")]
    #[diagnostic(code(cli::import_error), help("Check the file name."))]
    Import(String),
    #[error("input error: {0}")]
    #[diagnostic(code(cli::input_error))]
    Input(String),
    #[error("io error")]
    #[diagnostic(code(cli::io_error))]
    IO(#[from] std::io::Error),
    #[error("internal error")]
    Internal(#[source] SourceError),
    #[error("keystore error")]
    #[diagnostic(code(cli::keystore_error))]
    KeyStore(#[source] KeyStoreError),
    #[error("missing flag: {0}")]
    #[diagnostic(code(cli::config_error), help("Check the configuration file format."))]
    MissingFlag(String),
    #[error("network id error")]
    NetworkIdError(#[from] NetworkIdError),
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    #[error("parse error: {1}")]
    #[diagnostic(code(cli::parse_error), help("Check the inputs."))]
    Parse(#[source] SourceError, String),
    #[error("script builder error")]
    #[diagnostic(code(cli::script_builder_error))]
    ScriptBuilder(#[from] ScriptBuilderError),
    #[error("transaction error: {1}")]
    #[diagnostic(code(cli::transaction_error))]
    Transaction(#[source] SourceError, String),
}

/// Formats `ClientError` with special handling for RPC version mismatch errors.
fn format_client_error(client_error: &ClientError) -> String {
    match client_error {
        ClientError::RpcError(RpcError::RpcVersionMismatch { client_version, server_version }) => {
            let server_info = match server_version {
                Some(version) => format!("server version '{version}'"),
                None => "an incompatible server version".to_string(),
            };

            format!(
                "RPC version mismatch: Your client (version '{client_version}') is incompatible with {server_info}.\n\n\
                This usually happens when:\n\
                • Your client is newer than the server - use an older client version\n\
                • Your client is older than the server - update your client\n\
                • You're connecting to a server with a different protocol version\n\n\
                Please update your client or connect to a compatible server."
            )
        },
        _ => format!("client error: {client_error}"),
    }
}

#[cfg(test)]
mod tests {
    use miden_client::{ClientError, rpc::RpcError};

    use super::*;

    #[test]
    fn test_format_client_error_version_mismatch() {
        let version_mismatch_error = ClientError::RpcError(RpcError::RpcVersionMismatch {
            client_version: "0.11.0".to_string(),
            server_version: Some("0.10.0".to_string()),
        });

        let formatted = format_client_error(&version_mismatch_error);
        assert!(formatted.contains("RPC version mismatch"));
        assert!(formatted.contains("0.11.0"));
        assert!(formatted.contains("0.10.0"));

        // Test with AccountId for other error types
        let account_id =
            miden_objects::account::AccountId::try_from(0x1234_5678_90ab_cdef_u128).unwrap();
        let other_error = ClientError::AccountDataNotFound(account_id);
        let formatted_other = format_client_error(&other_error);
        assert!(formatted_other.contains("client error:"));
    }
}
