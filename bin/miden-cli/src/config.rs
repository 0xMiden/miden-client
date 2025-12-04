use core::fmt::Debug;
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

use figment::value::{Dict, Map};
use figment::{Metadata, Profile, Provider};
use miden_client::note_transport::NOTE_TRANSPORT_DEFAULT_ENDPOINT;
use miden_client::rpc::Endpoint;
use serde::{Deserialize, Serialize};

use crate::errors::CliError;

pub const MIDEN_DIR: &str = ".miden";
pub const TOKEN_SYMBOL_MAP_FILENAME: &str = "token_symbol_map.toml";
pub const DEFAULT_PACKAGES_DIR: &str = "packages";
pub const STORE_FILENAME: &str = "store.sqlite3";
pub const KEYSTORE_DIRECTORY: &str = "keystore";
pub const DEFAULT_TESTNET_FAUCET_API_URL: &str = "https://faucet-api.testnet.miden.io";
pub const DEFAULT_DEVNET_FAUCET_API_URL: &str = "https://faucet-api.devnet.miden.io";

/// Returns the global miden directory path in the user's home directory
pub fn get_global_miden_dir() -> Result<PathBuf, std::io::Error> {
    dirs::home_dir()
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "Could not determine home directory")
        })
        .map(|home| home.join(MIDEN_DIR))
}

/// Returns the local miden directory path relative to the current working directory
pub fn get_local_miden_dir() -> Result<PathBuf, std::io::Error> {
    std::env::current_dir().map(|cwd| cwd.join(MIDEN_DIR))
}

// CLI CONFIG
// ================================================================================================

#[derive(Debug, Deserialize, Serialize)]
pub struct CliConfig {
    /// Describes settings related to the RPC endpoint.
    pub rpc: RpcConfig,
    /// Settings related to the faucet API endpoint.
    #[serde(default)]
    pub faucet: FaucetConfig,
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
        // Create paths relative to the config file location (which is in .miden directory)
        // These will be resolved relative to the .miden directory when the config is loaded
        Self {
            rpc: RpcConfig::default(),
            faucet: FaucetConfig::default(),
            store_filepath: PathBuf::from(STORE_FILENAME),
            secret_keys_directory: PathBuf::from(KEYSTORE_DIRECTORY),
            token_symbol_map_filepath: PathBuf::from(TOKEN_SYMBOL_MAP_FILENAME),
            remote_prover_endpoint: None,
            package_directory: PathBuf::from(DEFAULT_PACKAGES_DIR),
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

// FAUCET CONFIG
// ================================================================================================

/// Default timeout for faucet requests in milliseconds.
///
/// Note: This must be a module-level function (not a method in an impl block) because
/// `#[serde(default = "...")]` requires a string path that serde can resolve during macro
/// expansion. Method paths like `Self::method_name` cannot be used in this context.
fn default_faucet_timeout_ms() -> u64 {
    30_000
}

/// Settings for the faucet API client.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FaucetConfig {
    /// Optional override for the faucet API base URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    /// Timeout for faucet requests in milliseconds.
    #[serde(default = "default_faucet_timeout_ms")]
    pub timeout_ms: u64,
}

impl Default for FaucetConfig {
    fn default() -> Self {
        Self {
            endpoint: None,
            timeout_ms: default_faucet_timeout_ms(),
        }
    }
}

impl FaucetConfig {
    /// Returns the faucet endpoint corresponding to the provided RPC endpoint, unless a custom
    /// faucet endpoint was explicitly configured.
    pub fn resolve_endpoint(&self, rpc_endpoint: &Endpoint) -> String {
        let default_endpoint = default_faucet_endpoint_for_rpc(rpc_endpoint);

        match &self.endpoint {
            // Treat legacy configs that hard-coded the other network's faucet as "unset" so we
            // transparently switch to the faucet that matches the current RPC endpoint.
            Some(configured) if is_other_network_default(rpc_endpoint, configured) => {
                default_endpoint.to_string()
            },
            Some(configured) => configured.clone(),
            None => default_endpoint.to_string(),
        }
    }
}

fn default_faucet_endpoint_for_rpc(rpc_endpoint: &Endpoint) -> &'static str {
    if rpc_endpoint == &Endpoint::devnet() {
        DEFAULT_DEVNET_FAUCET_API_URL
    } else {
        DEFAULT_TESTNET_FAUCET_API_URL
    }
}

fn is_other_network_default(rpc_endpoint: &Endpoint, configured: &str) -> bool {
    (rpc_endpoint == &Endpoint::devnet() && configured == DEFAULT_TESTNET_FAUCET_API_URL)
        || (rpc_endpoint == &Endpoint::testnet() && configured == DEFAULT_DEVNET_FAUCET_API_URL)
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
        let endpoint = Endpoint::try_from(endpoint).map_err(|err| err.clone())?;
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
