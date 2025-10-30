use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;
use tracing::info;

use crate::config::{CliConfig, CliEndpoint, Network, NoteTransportConfig};
use crate::errors::CliError;

/// Contains the account component template file generated on build.rs, corresponding to the basic
/// wallet component.
const BASIC_WALLET_PACKAGE: (&str, &[u8]) = (
    "basic-wallet.masp",
    include_bytes!(concat!(env!("OUT_DIR"), "/packages/", "basic-wallet.masp")),
);

/// Contains the account component template file generated on build.rs, corresponding to the
/// fungible faucet component.
const FAUCET_PACKAGE: (&str, &[u8]) = (
    "basic-fungible-faucet.masp",
    include_bytes!(concat!(env!("OUT_DIR"), "/packages/", "basic-fungible-faucet.masp")),
);

/// Contains the account component template file generated on build.rs, corresponding to the basic
/// auth component.
const BASIC_AUTH_PACKAGE: (&str, &[u8]) = (
    "auth/basic-auth.masp",
    include_bytes!(concat!(env!("OUT_DIR"), "/packages/auth/", "basic-auth.masp")),
);

/// Contains the account component template file generated on build.rs, corresponding to the no-auth
/// component.
const NO_AUTH_PACKAGE: (&str, &[u8]) = (
    "auth/no-auth.masp",
    include_bytes!(concat!(env!("OUT_DIR"), "/packages/auth/", "no-auth.masp")),
);

/// Contains the account component template file generated on build.rs, corresponding to the
/// multisig auth component.
const MULTISIG_AUTH_PACKAGE: (&str, &[u8]) = (
    "auth/multisig-auth.masp",
    include_bytes!(concat!(env!("OUT_DIR"), "/packages/auth/", "multisig-auth.masp")),
);

/// Contains the account component template file generated on build.rs, corresponding to the
/// ACL auth component.
const ACL_AUTH_PACKAGE: (&str, &[u8]) = (
    "auth/acl-auth.masp",
    include_bytes!(concat!(env!("OUT_DIR"), "/packages/auth/", "acl-auth.masp")),
);

const DEFAULT_INCLUDED_PACKAGES: [(&str, &[u8]); 6] = [
    BASIC_WALLET_PACKAGE,
    FAUCET_PACKAGE,
    BASIC_AUTH_PACKAGE,
    NO_AUTH_PACKAGE,
    MULTISIG_AUTH_PACKAGE,
    ACL_AUTH_PACKAGE,
];

// INIT COMMAND
// ================================================================================================

#[derive(Debug, Clone, Parser, Default)]
#[command(about = "Initialize the client. It will create a `.miden` directory with a \
`miden-client.toml` file that holds the CLI and client configurations")]
pub struct InitCmd {
    /// Network configuration to use. Options are `devnet`, `testnet`, `localhost` or a custom RPC
    /// endpoint. By default, the command uses the Testnet network.
    #[clap(long, short)]
    network: Option<Network>,

    /// Path to the store file.
    #[arg(long)]
    store_path: Option<String>,

    /// RPC endpoint for the remote prover. Required if proving mode is set to remote.
    /// The endpoint must be in the form of "{protocol}://{hostname}:{port}", being the protocol
    /// and port optional.
    /// If the proving RPC isn't set, the proving mode will be set to local.
    #[arg(long)]
    remote_prover_endpoint: Option<String>,

    /// RPC endpoint for the note transport node. Required to use the note transport network to
    /// exchange private notes.
    /// The endpoint must be in the form of "{protocol}://{hostname}:{port}", being the protocol
    /// and port optional.
    #[arg(long)]
    note_transport_endpoint: Option<String>,

    /// Maximum number of blocks the client can be behind the network.
    #[clap(long)]
    block_delta: Option<u32>,
}

impl InitCmd {
    pub fn execute(&self, config_file_path: &PathBuf) -> Result<(), CliError> {
        if config_file_path.exists() {
            return Err(CliError::Config(
                "Error with the configuration file".to_string().into(),
                format!(
                    "The file \"{:?}\" already exists in the working directory. Please try using another directory or removing the file.",
                    config_file_path.display(),
                ),
            ));
        }

        // Create the .miden directory if it doesn't exist
        if let Some(parent_dir) = config_file_path.parent() {
            fs::create_dir_all(parent_dir).map_err(|err| {
                CliError::Config(
                    Box::new(err),
                    format!("failed to create .miden directory in {}", parent_dir.display()),
                )
            })?;
        }

        let mut cli_config = CliConfig::default();

        if let Some(network) = &self.network {
            cli_config.rpc.endpoint = CliEndpoint::try_from(network.clone())?;
        }

        if let Some(path) = &self.store_path {
            cli_config.store_filepath = PathBuf::from(path);
        }

        cli_config.remote_prover_endpoint = match &self.remote_prover_endpoint {
            Some(rpc) => CliEndpoint::try_from(rpc.as_str()).ok(),
            None => None,
        };

        cli_config.note_transport =
            self.note_transport_endpoint.as_ref().map(|rpc| NoteTransportConfig {
                endpoint: rpc.clone(),
                ..Default::default()
            });

        cli_config.max_block_number_delta = self.block_delta;

        let config_as_toml_string = toml::to_string_pretty(&cli_config).map_err(|err| {
            CliError::Config("failed to serialize config".to_string().into(), err.to_string())
        })?;

        let mut file_handle = File::options()
            .write(true)
            .create_new(true)
            .open(config_file_path)
            .map_err(|err| {
            CliError::Config("failed to create config file".to_string().into(), err.to_string())
        })?;

        // Resolve package directory relative to .miden directory before writing files
        let config_dir = config_file_path.parent().unwrap();
        let resolved_package_dir = if cli_config.package_directory.is_relative() {
            config_dir.join(&cli_config.package_directory)
        } else {
            cli_config.package_directory.clone()
        };
        write_packages_files(&resolved_package_dir)?;

        file_handle.write(config_as_toml_string.as_bytes()).map_err(|err| {
            CliError::Config("failed to write config file".to_string().into(), err.to_string())
        })?;

        println!("Config file successfully created at: {}", config_file_path.display());

        Ok(())
    }
}

/// Creates the directory specified by `packages_dir` and writes the `DEFAULT_INCLUDED_PACKAGES`.
fn write_packages_files(packages_dir: &PathBuf) -> Result<(), CliError> {
    fs::create_dir_all(packages_dir).map_err(|err| {
        CliError::Config(
            Box::new(err),
            "failed to create account component templates directory".into(),
        )
    })?;

    for package in DEFAULT_INCLUDED_PACKAGES {
        let package_path = packages_dir.join(package.0);
        let mut lib_file = File::create(&package_path).map_err(|err| {
            CliError::Config(
                Box::new(err),
                format!("Failed to create file at {}", package_path.display()),
            )
        })?;
        lib_file.write_all(package.1).map_err(|err| {
            CliError::Config(
                Box::new(err),
                format!(
                    "Failed to write package {} into file {}",
                    package.0,
                    package_path.display()
                ),
            )
        })?;
    }

    info!("Packages files successfully created in: {:?}", packages_dir);

    Ok(())
}
