use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;
use tracing::info;

use crate::CLIENT_CONFIG_FILE_NAME;
use crate::config::{
    CliConfig,
    CliEndpoint,
    MIDEN_DIR,
    Network,
    NoteTransportConfig,
    get_global_miden_dir,
    get_local_miden_dir,
};
use crate::errors::CliError;

const PACKAGES_DIR: &str = "packages";

// INIT COMMAND
// ================================================================================================

#[derive(Debug, Clone, Parser, Default)]
#[command(
    about = "Initialize the client. By default creates a global `.miden` directory in the home directory. \
Use --local to create a local `.miden` directory in the current working directory."
)]
pub struct InitCmd {
    /// Create configuration in the local working directory instead of the global home directory
    #[clap(long)]
    local: bool,

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
    pub fn execute(&self) -> Result<(), CliError> {
        // Determine target directory based on flags
        let (target_miden_dir, config_type) = if self.local {
            (get_local_miden_dir()?, "local")
        } else {
            (
                get_global_miden_dir().map_err(|e| {
                    CliError::Config(Box::new(e), "Failed to determine home directory".to_string())
                })?,
                "global",
            )
        };

        let config_file_path = target_miden_dir.join(CLIENT_CONFIG_FILE_NAME);

        // Check if config already exists
        if config_file_path.exists() {
            return Err(CliError::Config(
                "Error with the configuration file".to_string().into(),
                format!(
                    "The file \"{}\" already exists in the {} {} directory ({}). Please remove it first or use a different location.",
                    CLIENT_CONFIG_FILE_NAME,
                    config_type,
                    MIDEN_DIR,
                    target_miden_dir.display()
                ),
            ));
        }

        // Create the miden directory if not existent
        fs::create_dir_all(&target_miden_dir).map_err(|err| {
            CliError::Config(
                Box::new(err),
                format!(
                    "failed to create {} {} directory in {}",
                    config_type,
                    MIDEN_DIR,
                    target_miden_dir.display()
                ),
            )
        })?;

        // Create new config for target directory
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
            .open(&config_file_path)
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

        file_handle.write_all(config_as_toml_string.as_bytes()).map_err(|err| {
            CliError::Config("failed to write config file".to_string().into(), err.to_string())
        })?;

        println!(
            "Config file successfully created at: {} ({})",
            config_file_path.display(),
            config_type
        );

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

    let build_packages_dir = PathBuf::from(env!("OUT_DIR")).join(PACKAGES_DIR);

    let packages = collect_packages(&build_packages_dir)?;

    // Write each package file to the destination directory
    for (relative_path, contents) in packages {
        let package_path = packages_dir.join(&relative_path);

        if let Some(parent) = package_path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                CliError::Config(
                    Box::new(err),
                    format!("Failed to create directory {}", parent.display()),
                )
            })?;
        }

        let mut lib_file = File::create(&package_path).map_err(|err| {
            CliError::Config(
                Box::new(err),
                format!("Failed to create file at {}", package_path.display()),
            )
        })?;
        lib_file.write_all(&contents).map_err(|err| {
            CliError::Config(
                Box::new(err),
                format!(
                    "Failed to write package {} into file {}",
                    relative_path.display(),
                    package_path.display()
                ),
            )
        })?;
    }

    info!("Packages files successfully created in: {:?}", packages_dir);

    Ok(())
}

fn visit_dir(
    dir: &PathBuf,
    base_dir: &PathBuf,
    packages: &mut Vec<(PathBuf, Vec<u8>)>,
) -> Result<(), CliError> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            visit_dir(&path, base_dir, packages)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("masp") {
            let contents = fs::read(&path)?;

            let relative_path = path
                .strip_prefix(base_dir)
                .expect("Path should be under base directory")
                .to_path_buf();

            packages.push((relative_path, contents));
        }
    }

    Ok(())
}

/// Recursively collects all .masp files from the packages directory built during build.rs.
/// Returns a vector of tuples containing the relative path and file contents.
fn collect_packages(packages_dir: &PathBuf) -> Result<Vec<(PathBuf, Vec<u8>)>, CliError> {
    let mut packages = Vec::new();

    visit_dir(packages_dir, packages_dir, &mut packages)?;

    Ok(packages)
}
