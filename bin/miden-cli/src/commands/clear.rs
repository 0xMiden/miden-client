use std::fs;
use std::path::PathBuf;

use clap::Parser;

use crate::config::{MIDEN_DIR, get_global_miden_dir, get_local_miden_dir};
use crate::errors::CliError;

// CLEAR COMMAND
// ================================================================================================

#[derive(Debug, Clone, Parser)]
#[command(
    about = "Clear miden client configuration. By default removes local config if present, otherwise removes global config. \
Use --global to specifically target global config."
)]
pub struct ClearCmd {
    /// Force removal of global configuration, even if local config exists
    #[clap(long)]
    global: bool,
}

impl ClearCmd {
    pub fn execute(&self) -> Result<(), CliError> {
        if self.global {
            // Clear global config specifically
            Self::clear_global_config()
        } else {
            // Priority logic: local first, then global
            Self::clear_with_priority()
        }
    }

    fn clear_with_priority() -> Result<(), CliError> {
        // Try local config first
        let local_miden_dir = get_local_miden_dir()?;
        if local_miden_dir.exists() {
            Self::remove_directory(&local_miden_dir, "local")?;
            return Ok(());
        }

        // Fallback to global config
        Self::clear_global_config()?;
        Ok(())
    }

    fn clear_global_config() -> Result<(), CliError> {
        let global_miden_dir = get_global_miden_dir().map_err(|e| {
            CliError::Config(Box::new(e), "Failed to determine home directory".to_string())
        })?;

        if global_miden_dir.exists() {
            Self::remove_directory(&global_miden_dir, "global")?;
        } else {
            println!("No global miden configuration found to clear.");
        }

        Ok(())
    }

    fn remove_directory(dir_path: &PathBuf, config_type: &str) -> Result<(), CliError> {
        println!("Removing {config_type} miden configuration at: {}", dir_path.display());

        fs::remove_dir_all(dir_path).map_err(|err| {
            CliError::Config(
                Box::new(err),
                format!("Failed to remove {config_type} {MIDEN_DIR} directory"),
            )
        })?;

        println!("Successfully removed {config_type} miden configuration.");
        Ok(())
    }
}
