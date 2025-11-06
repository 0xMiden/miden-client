use core::fmt::Debug;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use figment::value::{Dict, Map};
use figment::{Metadata, Profile, Provider};
use miden_client::note_transport::NOTE_TRANSPORT_DEFAULT_ENDPOINT;
use miden_client::rpc::Endpoint;
use serde::{Deserialize, Serialize};

use crate::errors::CliError;

const TOKEN_SYMBOL_MAP_FILEPATH: &str = "token_symbol_map.toml";
const DEFAULT_PACKAGES_DIR: &str = "./packages";

// CLI CONFIG
// ================================================================================================

#[derive(Debug, Deserialize, Serialize)]
pub struct CliConfig {
    /// Describes settings related to the RPC endpoint.
    pub rpc: RpcConfig,
    /// Path to the `SQLite` store file.
    pub store_filepath: PathBuf,
    /// Path to the directory that contains the secret key files.
    pub secret_keys_directory: PathBuf,
    /// Path to the file containing the token symbol map.
    pub token_symbol_map_filepath: PathBuf,
    /// RPC endpoint for the remote prover. If this isn't present, a local prover will be used.
    pub remote_prover_endpoint: Option<CliEndpoint>,
    /// Path to the directory from where packages will be loaded.
    pub package_directory: PathBuf,
    /// Maximum number of blocks the client can be behind the network for transactions and account
    /// proofs to be considered valid.
    pub max_block_number_delta: Option<u32>,
    /// Describes settings related to the note transport endpoint.
    pub note_transport: Option<NoteTransportConfig>,
}

// Make `ClientConfig` a provider itself for composability.
impl Provider for CliConfig {
    fn metadata(&self) -> Metadata {
        Metadata::named("CLI Config")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, figment::Error> {
        figment::providers::Serialized::defaults(CliConfig::default()).data()
    }

    fn profile(&self) -> Option<Profile> {
        // Optionally, a profile that's selected by default.
        None
    }
}

impl Default for CliConfig {
    fn default() -> Self {
        const STORE_FILENAME: &str = "store.sqlite3";
        const KEYSTORE_DIRECTORY: &str = "keystore";

        // Get current directory
        let exec_dir = PathBuf::new();

        Self {
            rpc: RpcConfig::default(),
            store_filepath: exec_dir.join(STORE_FILENAME),
            secret_keys_directory: exec_dir.join(KEYSTORE_DIRECTORY),
            token_symbol_map_filepath: Path::new(TOKEN_SYMBOL_MAP_FILEPATH).to_path_buf(),
            remote_prover_endpoint: None,
            package_directory: Path::new(DEFAULT_PACKAGES_DIR).to_path_buf(),
            max_block_number_delta: None,
            note_transport: None,
        }
    }
}

// RPC CONFIG
// ================================================================================================

/// Settings for the RPC client.
#[derive(Debug, Deserialize, Serialize)]
pub struct RpcConfig {
    /// Address of the Miden node to connect to.
    pub endpoint: CliEndpoint,
    /// Timeout for the RPC api requests, in milliseconds.
    pub timeout_ms: u64,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            endpoint: Endpoint::testnet().into(),
            timeout_ms: 10000,
        }
    }
}

// NOTE TRANSPORT CONFIG
// ================================================================================================

/// Settings for the note transport client.
#[derive(Debug, Deserialize, Serialize)]
pub struct NoteTransportConfig {
    /// Address of the Miden Note Transport node to connect to.
    pub endpoint: String,
    /// Timeout for the Note Transport RPC api requests, in milliseconds.
    pub timeout_ms: u64,
}

impl Default for NoteTransportConfig {
    fn default() -> Self {
        Self {
            endpoint: NOTE_TRANSPORT_DEFAULT_ENDPOINT.to_string(),
            timeout_ms: 10000,
        }
    }
}

// CLI ENDPOINT
// ================================================================================================

#[derive(Clone, Debug)]
pub struct CliEndpoint(pub Endpoint);

impl Display for CliEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for CliEndpoint {
    type Error = String;

    fn try_from(endpoint: &str) -> Result<Self, Self::Error> {
        let endpoint = Endpoint::try_from(endpoint).map_err(|err| err.to_string())?;
        Ok(Self(endpoint))
    }
}

impl From<Endpoint> for CliEndpoint {
    fn from(endpoint: Endpoint) -> Self {
        Self(endpoint)
    }
}

impl TryFrom<Network> for CliEndpoint {
    type Error = CliError;

    fn try_from(value: Network) -> Result<Self, Self::Error> {
        Ok(Self(Endpoint::try_from(value.to_rpc_endpoint().as_str()).map_err(|err| {
            CliError::Parse(err.into(), "Failed to parse RPC endpoint".to_string())
        })?))
    }
}

impl From<CliEndpoint> for Endpoint {
    fn from(endpoint: CliEndpoint) -> Self {
        endpoint.0
    }
}

impl From<&CliEndpoint> for Endpoint {
    fn from(endpoint: &CliEndpoint) -> Self {
        endpoint.0.clone()
    }
}

impl Serialize for CliEndpoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for CliEndpoint {
    fn deserialize<D>(deserializer: D) -> Result<CliEndpoint, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let endpoint = String::deserialize(deserializer)?;
        CliEndpoint::try_from(endpoint.as_str()).map_err(serde::de::Error::custom)
    }
}

// NETWORK
// ================================================================================================

/// Represents the network to which the client connects. It is used to determine the RPC endpoint
/// and network ID for the CLI.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Network {
    Custom(String),
    Devnet,
    Localhost,
    Testnet,
}

impl FromStr for Network {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "devnet" => Ok(Network::Devnet),
            "localhost" => Ok(Network::Localhost),
            "testnet" => Ok(Network::Testnet),
            custom => Ok(Network::Custom(custom.to_string())),
        }
    }
}

impl Network {
    /// Converts the Network variant to its corresponding RPC endpoint string
    #[allow(dead_code)]
    pub fn to_rpc_endpoint(&self) -> String {
        match self {
            Network::Custom(custom) => custom.clone(),
            Network::Devnet => Endpoint::devnet().to_string(),
            Network::Localhost => Endpoint::default().to_string(),
            Network::Testnet => Endpoint::testnet().to_string(),
        }
    }
}
