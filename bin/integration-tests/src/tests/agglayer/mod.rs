use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use miden_client::Deserializable;
use miden_client::account::{AccountFile, AccountId};
use miden_client::keystore::Keystore;
use miden_client::testing::common::FilesystemKeyStore;

pub mod ger;

/// Configuration for agglayer tests when running against a node with pre-deployed
/// agglayer accounts (e.g. complete genesis or devnet).
///
/// Loaded from `.mac` files in the directory specified by `AGGLAYER_ACCOUNTS_DIR` env var.
pub struct AgglayerConfig {
    pub bridge_admin: AccountFile,
    pub ger_manager: AccountFile,
    pub bridge: AccountFile,
    pub faucet: AccountFile,
}

impl AgglayerConfig {
    /// File names matching the node-builder output.
    const BRIDGE_ADMIN_FILE: &str = "bridge_admin.mac";
    const GER_MANAGER_FILE: &str = "ger_manager.mac";
    const BRIDGE_FILE: &str = "bridge.mac";
    const FAUCET_FILE: &str = "agglayer_faucet.mac";

    /// Loads agglayer config from the given directory containing `.mac` files.
    pub fn load_from_dir(dir: &Path) -> Result<Self> {
        let bridge_admin = Self::load_account_file(dir, Self::BRIDGE_ADMIN_FILE)?;
        let ger_manager = Self::load_account_file(dir, Self::GER_MANAGER_FILE)?;
        let bridge = Self::load_account_file(dir, Self::BRIDGE_FILE)?;
        let faucet = Self::load_account_file(dir, Self::FAUCET_FILE)?;

        Ok(Self {
            bridge_admin,
            ger_manager,
            bridge,
            faucet,
        })
    }

    /// Tries to load agglayer config from the `AGGLAYER_ACCOUNTS_DIR` env var.
    /// Returns `None` if the env var is not set.
    pub fn from_env() -> Result<Option<Self>> {
        match std::env::var("AGGLAYER_ACCOUNTS_DIR") {
            Ok(dir) => Ok(Some(Self::load_from_dir(&PathBuf::from(dir))?)),
            Err(_) => Ok(None),
        }
    }

    #[allow(dead_code)]
    pub fn bridge_admin_id(&self) -> AccountId {
        self.bridge_admin.account.id()
    }

    pub fn ger_manager_id(&self) -> AccountId {
        self.ger_manager.account.id()
    }

    pub fn bridge_id(&self) -> AccountId {
        self.bridge.account.id()
    }

    #[allow(dead_code)]
    pub fn faucet_id(&self) -> AccountId {
        self.faucet.account.id()
    }

    /// Imports all agglayer accounts and their secret keys into the client and keystore.
    pub async fn import_into_client(
        &self,
        client: &mut miden_client::testing::common::TestClient,
        keystore: &FilesystemKeyStore,
    ) -> Result<()> {
        for account_file in [&self.bridge_admin, &self.ger_manager, &self.bridge, &self.faucet] {
            client.add_account(&account_file.account, false).await.with_context(|| {
                format!("failed to add account {} to client", account_file.account.id())
            })?;

            for secret_key in &account_file.auth_secret_keys {
                keystore.add_key(secret_key, account_file.account.id()).await.with_context(
                    || {
                        format!(
                            "failed to add key for account {} to keystore",
                            account_file.account.id()
                        )
                    },
                )?;
            }
        }

        Ok(())
    }

    fn load_account_file(dir: &Path, filename: &str) -> Result<AccountFile> {
        let path = dir.join(filename);
        let bytes =
            std::fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
        AccountFile::read_from_bytes(&bytes)
            .map_err(|e| anyhow::anyhow!("failed to deserialize {}: {}", path.display(), e))
    }
}
