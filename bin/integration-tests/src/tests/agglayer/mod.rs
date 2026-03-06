use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use miden_agglayer::{AggLayerFaucet, EthAddressFormat, create_bridge_account};
use miden_client::Deserializable;
use miden_client::account::{AccountFile, AccountId, AccountStorageMode};
use miden_client::auth::RPO_FALCON_SCHEME_ID;
use miden_client::crypto::FeltRng;
use miden_client::keystore::Keystore;
use miden_client::testing::common::{
    FilesystemKeyStore,
    TestClient,
    insert_new_wallet,
    wait_for_node,
    wait_for_tx,
};
use miden_client::transaction::TransactionRequestBuilder;

use crate::tests::config::ClientConfig;

pub mod agglayer_bridge_in_out;
mod agglayer_test_utils;
pub mod ger;

// AGGLAYER CONFIG
// ================================================================================================

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

    /// Tries to load agglayer config from the `AGGLAYER_ACCOUNTS_DIR` env var.
    /// Returns `None` if the env var is not set.
    pub fn from_env() -> Result<Option<Self>> {
        match std::env::var("AGGLAYER_ACCOUNTS_DIR") {
            Ok(dir) => {
                let dir = PathBuf::from(dir);
                let bridge_admin = Self::load_account_file(&dir, Self::BRIDGE_ADMIN_FILE)?;
                let ger_manager = Self::load_account_file(&dir, Self::GER_MANAGER_FILE)?;
                let bridge = Self::load_account_file(&dir, Self::BRIDGE_FILE)?;
                let faucet = Self::load_account_file(&dir, Self::FAUCET_FILE)?;
                Ok(Some(Self {
                    bridge_admin,
                    ger_manager,
                    bridge,
                    faucet,
                }))
            },
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

        let felts = [info1[0], info1[1], info1[2], info1[3], info2[0]];
        let mut bytes = [0u8; 20];
        for (i, felt) in felts.iter().enumerate() {
            let val = felt.as_int() as u32;
            bytes[i * 4..(i + 1) * 4].copy_from_slice(&val.to_le_bytes());
        }
        EthAddressFormat::new(bytes)
    }

    /// Returns the faucet's origin network from its storage.
    #[allow(dead_code)]
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

// SHARED TEST SETUP
// ================================================================================================

/// A client + keystore pair.
pub type ClientWithKeystore = (TestClient, FilesystemKeyStore);

/// Account IDs produced by the core setup: `(bridge_admin_id, ger_manager_id, bridge_id)`.
pub type CoreAccountIds = (AccountId, AccountId, AccountId);

/// Creates three clients sharing the same RPC endpoint, for bridge admin, GER manager, and user.
pub async fn create_agglayer_clients(
    client_config: &ClientConfig,
) -> Result<(ClientWithKeystore, ClientWithKeystore, ClientWithKeystore)> {
    let (mut bridge_admin_client, bridge_admin_keystore) =
        client_config.clone().into_client().await?;
    wait_for_node(&mut bridge_admin_client).await;
    bridge_admin_client.sync_state().await?;
    println!("[setup] Bridge admin client initialized");

    let ger_manager = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;
    println!("[setup] GER manager client initialized");

    let user = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;
    println!("[setup] User client initialized");

    Ok(((bridge_admin_client, bridge_admin_keystore), ger_manager, user))
}

/// Sets up the core agglayer accounts (bridge admin, GER manager, bridge) across 3 clients.
///
/// In genesis mode, imports accounts from the network into the appropriate clients.
/// In runtime mode, creates, deploys, and distributes accounts.
pub async fn setup_core_accounts(
    config: Option<&AgglayerConfig>,
    bridge_admin: &mut ClientWithKeystore,
    ger_manager: &mut ClientWithKeystore,
    user: &mut ClientWithKeystore,
) -> Result<CoreAccountIds> {
    match config {
        Some(config) => {
            println!("[setup] Loading core accounts from genesis");
            println!("[setup]   bridge admin:  {}", config.bridge_admin_id());
            println!("[setup]   GER manager:   {}", config.ger_manager_id());
            println!("[setup]   bridge:        {}", config.bridge_id());

            config
                .import_account(config.bridge_admin_id(), &mut bridge_admin.0, &bridge_admin.1)
                .await?;
            config
                .import_account(config.ger_manager_id(), &mut ger_manager.0, &ger_manager.1)
                .await?;

            for (client, keystore) in [&mut *bridge_admin, &mut *ger_manager, &mut *user] {
                config.import_account(config.bridge_id(), client, keystore).await?;
            }

            Ok((config.bridge_admin_id(), config.ger_manager_id(), config.bridge_id()))
        },
        None => {
            println!("[setup] Creating core accounts at runtime");

            let (bridge_admin_account, ..) = insert_new_wallet(
                &mut bridge_admin.0,
                AccountStorageMode::Private,
                &bridge_admin.1,
                RPO_FALCON_SCHEME_ID,
            )
            .await?;

            let (ger_manager_account, ..) = insert_new_wallet(
                &mut ger_manager.0,
                AccountStorageMode::Private,
                &ger_manager.1,
                RPO_FALCON_SCHEME_ID,
            )
            .await?;

            let bridge_account = create_bridge_account(
                bridge_admin.0.rng().draw_word(),
                bridge_admin_account.id(),
                ger_manager_account.id(),
            );
            println!("[setup]   bridge admin:  {}", bridge_admin_account.id());
            println!("[setup]   GER manager:   {}", ger_manager_account.id());
            println!("[setup]   bridge:        {}", bridge_account.id());

            bridge_admin.0.add_account(&bridge_account, false).await?;
            ger_manager.0.add_account(&bridge_account, false).await?;
            user.0.add_account(&bridge_account, false).await?;

            let deploy_tx = TransactionRequestBuilder::new().build()?;
            let tx_id =
                bridge_admin.0.submit_new_transaction(bridge_account.id(), deploy_tx).await?;
            wait_for_tx(&mut bridge_admin.0, tx_id).await?;
            println!("[setup] Bridge account deployed on-chain");

            Ok((bridge_admin_account.id(), ger_manager_account.id(), bridge_account.id()))
        },
    }
}
