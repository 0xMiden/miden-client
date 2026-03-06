use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use miden_agglayer::{AggLayerFaucet, EthAddressFormat};
use miden_client::account::{AccountFile, AccountId};
use miden_client::keystore::Keystore;
use miden_client::testing::common::{FilesystemKeyStore, TestClient};
use miden_client::Deserializable;

pub mod agglayer_bridge_in_out;
mod agglayer_test_utils;
pub mod ger;

/// Configuration for agglayer tests when running against a node with pre-deployed
/// agglayer accounts (e.g. complete genesis or devnet).
///
/// Loaded from `.mac` files in the directory specified by `AGGLAYER_ACCOUNTS_DIR` env var.
/// Account IDs and keys are read from files, but the actual account state is fetched
/// from the network to ensure it's up-to-date (idempotent across repeated runs).
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

    pub fn bridge_admin_id(&self) -> AccountId {
        self.bridge_admin.account.id()
    }

    pub fn ger_manager_id(&self) -> AccountId {
        self.ger_manager.account.id()
    }

    pub fn bridge_id(&self) -> AccountId {
        self.bridge.account.id()
    }

    pub fn faucet_id(&self) -> AccountId {
        self.faucet.account.id()
    }

    /// Imports all agglayer accounts into the client by fetching the latest state from the
    /// network. Secret keys are loaded from the `.mac` files and added to the keystore.
    ///
    /// This ensures the client always has up-to-date account state, making tests idempotent
    /// even when run repeatedly against the same node.
    pub async fn import_into_client(
        &self,
        client: &mut TestClient,
        keystore: &FilesystemKeyStore,
    ) -> Result<()> {
        for account_file in [&self.bridge_admin, &self.ger_manager, &self.bridge, &self.faucet] {
            Self::import_account_file(account_file, client, keystore).await?;
        }
        Ok(())
    }

    /// Imports a single account (by ID) into the given client and keystore.
    /// Fetches the latest state from the network. Adds any matching secret keys.
    pub async fn import_account(
        &self,
        account_id: AccountId,
        client: &mut TestClient,
        keystore: &FilesystemKeyStore,
    ) -> Result<()> {
        let account_file = [&self.bridge_admin, &self.ger_manager, &self.bridge, &self.faucet]
            .into_iter()
            .find(|f| f.account.id() == account_id)
            .with_context(|| format!("account {account_id} not found in agglayer config"))?;

        Self::import_account_file(account_file, client, keystore).await
    }

    /// Returns the faucet's origin token address from its storage.
    pub fn faucet_origin_token_address(&self) -> EthAddressFormat {
        let info1 = self
            .faucet
            .account
            .storage()
            .get_item(AggLayerFaucet::conversion_info_1_slot())
            .expect("faucet should have conversion_info_1 slot");
        let info2 = self
            .faucet
            .account
            .storage()
            .get_item(AggLayerFaucet::conversion_info_2_slot())
            .expect("faucet should have conversion_info_2 slot");

        // Reconstruct 20-byte address from 5 Felts (4 from info1 + 1 from info2)
        // Each Felt holds a little-endian u32
        let felts = [info1[0], info1[1], info1[2], info1[3], info2[0]];
        let mut bytes = [0u8; 20];
        for (i, felt) in felts.iter().enumerate() {
            let val = felt.as_int() as u32;
            bytes[i * 4..(i + 1) * 4].copy_from_slice(&val.to_le_bytes());
        }
        EthAddressFormat::new(bytes)
    }

    /// Returns the faucet's origin network from its storage.
    pub fn faucet_origin_network(&self) -> u32 {
        let info2 = self
            .faucet
            .account
            .storage()
            .get_item(AggLayerFaucet::conversion_info_2_slot())
            .expect("faucet should have conversion_info_2 slot");
        info2[1].as_int() as u32
    }

    /// Returns the faucet's scale from its storage.
    pub fn faucet_scale(&self) -> u8 {
        let info2 = self
            .faucet
            .account
            .storage()
            .get_item(AggLayerFaucet::conversion_info_2_slot())
            .expect("faucet should have conversion_info_2 slot");
        info2[2].as_int() as u8
    }

    async fn import_account_file(
        account_file: &AccountFile,
        client: &mut TestClient,
        keystore: &FilesystemKeyStore,
    ) -> Result<()> {
        let account_id = account_file.account.id();

        client
            .import_account_by_id(account_id)
            .await
            .with_context(|| format!("failed to import account {account_id} from network"))?;

        for secret_key in &account_file.auth_secret_keys {
            keystore.add_key(secret_key, account_id).await.with_context(|| {
                format!("failed to add key for account {account_id} to keystore")
            })?;
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
