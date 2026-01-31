use std::path::PathBuf;
use std::{fs, io};

use clap::Parser;

use crate::config::{
    MIDEN_DIR,
    get_active_profile_from_env,
    get_global_miden_dir,
    get_local_miden_dir,
    get_profile_dir,
    list_profiles,
};
use crate::errors::CliError;

// CLEAR COMMAND
// ================================================================================================

#[derive(Debug, Clone, Parser)]
#[command(
    about = "Clear miden client configuration. By default removes local config if present, otherwise removes global config. \
Use --global to specifically target global config. Use --profile to target a specific profile."
)]
pub struct ClearConfigCmd {
    /// Force removal of global configuration, even if local config exists
    #[clap(long)]
    global: bool,
    /// Do not prompt for confirmation before deleting configuration directories
    #[clap(long)]
    force: bool,
    /// Profile name to clear (e.g., "testnet", "devnet").
    /// If not specified, uses MIDEN_PROFILE env var or clears the root config.
    #[clap(long, short)]
    profile: Option<String>,
    /// List all available profiles instead of clearing
    #[clap(long)]
    list: bool,
}

impl ClearConfigCmd {
    pub fn execute(&self) -> Result<(), CliError> {
        // Handle --list flag
        if self.list {
            return self.list_all_profiles();
        }

        // Determine the profile to use
        let profile = self.profile.clone().or_else(get_active_profile_from_env);

        if self.global {
            // Clear global config specifically
            self.clear_global_config(profile.as_deref())
        } else {
            // Priority logic: local first, then global
            self.try_clear_local_config(profile.as_deref())
        }
    }

    /// List all available profiles in both local and global directories.
    fn list_all_profiles(&self) -> Result<(), CliError> {
        println!("Available profiles:\n");

        // Check local profiles
        let local_miden_dir = get_local_miden_dir()?;
        if local_miden_dir.exists() {
            let local_profiles = list_profiles(&local_miden_dir).unwrap_or_default();
            println!("Local ({}):", local_miden_dir.display());
            if local_profiles.is_empty() {
                // Check if root config exists
                if local_miden_dir.join(crate::CLIENT_CONFIG_FILE_NAME).exists() {
                    println!("  (root) - default configuration");
                } else {
                    println!("  (none)");
                }
            } else {
                for profile in &local_profiles {
                    println!("  - {}", profile);
                }
                // Also check for root config
                if local_miden_dir.join(crate::CLIENT_CONFIG_FILE_NAME).exists() {
                    println!("  (root) - default configuration");
                }
            }
        } else {
            println!("Local: (not initialized)");
        }

        println!();

        // Check global profiles
        let global_miden_dir = get_global_miden_dir().map_err(|e| {
            CliError::Config(Box::new(e), "Failed to determine home directory".to_string())
        })?;
        if global_miden_dir.exists() {
            let global_profiles = list_profiles(&global_miden_dir).unwrap_or_default();
            println!("Global ({}):", global_miden_dir.display());
            if global_profiles.is_empty() {
                // Check if root config exists
                if global_miden_dir.join(crate::CLIENT_CONFIG_FILE_NAME).exists() {
                    println!("  (root) - default configuration");
                } else {
                    println!("  (none)");
                }
            } else {
                for profile in &global_profiles {
                    println!("  - {}", profile);
                }
                // Also check for root config
                if global_miden_dir.join(crate::CLIENT_CONFIG_FILE_NAME).exists() {
                    println!("  (root) - default configuration");
                }
            }
        } else {
            println!("Global: (not initialized)");
        }

        // Show current active profile
        if let Some(active) = get_active_profile_from_env() {
            println!("\nActive profile (MIDEN_PROFILE): {}", active);
        }

        Ok(())
    }

    /// Try to clear the local config if it exists, and if not, try to clear the global config.
    fn try_clear_local_config(&self, profile: Option<&str>) -> Result<(), CliError> {
        // Try local config first
        let local_miden_dir = get_local_miden_dir()?;
        let target_dir = get_profile_dir(&local_miden_dir, profile);

        if target_dir.exists() {
            let location_desc = match profile {
                Some(p) => format!("local profile '{}'", p),
                None => "local".to_string(),
            };
            self.remove_directory(&target_dir, &location_desc)?;
            return Ok(());
        }

        // Clear global config if no local config exists
        let profile_msg = profile.map(|p| format!(" for profile '{}'", p)).unwrap_or_default();
        println!(
            "No local configuration found{}. Attempting to clear global configuration.",
            profile_msg
        );
        self.clear_global_config(profile)
    }

    fn clear_global_config(&self, profile: Option<&str>) -> Result<(), CliError> {
        let global_miden_dir = get_global_miden_dir().map_err(|e| {
            CliError::Config(Box::new(e), "Failed to determine home directory".to_string())
        })?;
        let target_dir = get_profile_dir(&global_miden_dir, profile);

        let location_desc = match profile {
            Some(p) => format!("global profile '{}'", p),
            None => "global".to_string(),
        };

        if target_dir.exists() {
            self.remove_directory(&target_dir, &location_desc)?;
        } else {
            println!("No {} Miden configuration found to clear.", location_desc);
        }

        Ok(())
    }

    fn remove_directory(&self, dir_path: &PathBuf, config_type: &str) -> Result<(), CliError> {
        if !dir_path.exists() {
            println!("No {} Miden configuration found to clear.", config_type);
            return Ok(());
        }

        println!("Found {} Miden configuration at: {}", config_type, dir_path.display());

        if !self.force {
            println!("Are you sure you want to remove it? (y/N)");
            let mut proceed_str: String = String::new();
            io::stdin().read_line(&mut proceed_str)?;
            if proceed_str.trim().to_lowercase() != "y" {
                println!("Operation cancelled.");
                return Ok(());
            }
        }

        println!("Removing {} Miden configuration...", config_type);

        fs::remove_dir_all(dir_path).map_err(|err| {
            CliError::Config(
                Box::new(err),
                format!("Failed to remove {} {} directory", config_type, MIDEN_DIR),
            )
        })?;

        println!("Successfully removed {} miden configuration.", config_type);
        Ok(())
    }
}
