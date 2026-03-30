use anyhow::{Context, Result};
use miden_agglayer::{
    AggLayerBridge,
    EthAddressFormat,
    create_agglayer_faucet,
    create_bridge_account,
    faucet_registry_key,
};
use miden_node_utils::crypto::get_rpo_random_coin;
use miden_protocol::account::auth::{AuthScheme, AuthSecretKey};
use miden_protocol::account::{
    Account,
    AccountBuilder,
    AccountComponent,
    AccountComponentMetadata,
    AccountFile,
    AccountStorageMode,
    StorageMapKey,
};
use miden_protocol::{Felt, ONE, Word, ZERO};
use miden_standards::account::auth::AuthSingleSig;
use miden_standards::account::components::basic_wallet_library;
use rand::Rng;
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;

/// File names for agglayer genesis account exports.
pub const BRIDGE_ADMIN_ACCOUNT_FILE: &str = "bridge_admin.mac";
pub const GER_MANAGER_ACCOUNT_FILE: &str = "ger_manager.mac";
pub const BRIDGE_ACCOUNT_FILE: &str = "bridge.mac";
pub const AGGLAYER_FAUCET_ACCOUNT_FILE: &str = "agglayer_faucet.mac";

/// Deterministic test origin token address for the genesis faucet.
pub const TEST_ORIGIN_TOKEN_ADDRESS: [u8; 20] = [0xaa; 20];

/// Result of creating agglayer genesis accounts.
pub struct AgglayerGenesisAccounts {
    /// All accounts to include in genesis (with nonce = 1).
    pub accounts: Vec<Account>,
    /// Account files to save to disk (account + secret keys).
    /// Each entry is (filename, `AccountFile`).
    pub account_files: Vec<(&'static str, AccountFile)>,
}

/// Creates all agglayer genesis accounts:
/// 1. Bridge Admin - public wallet with Falcon512 auth
/// 2. GER Manager - public wallet with Falcon512 auth
/// 3. Bridge - network account (`NoAuth`) with the faucet pre-registered
/// 4. Faucet - network account (`NoAuth`) for bridged tokens
///
/// All accounts have their nonce set to ONE (genesis convention).
pub fn create_agglayer_genesis_accounts() -> Result<AgglayerGenesisAccounts> {
    let mut rng = ChaCha20Rng::from_seed(rand::random());

    // 1. Create Bridge Admin
    let admin_secret =
        AuthSecretKey::new_falcon512_rpo_with_rng(&mut get_rpo_random_coin(&mut rng));
    let admin_account = build_wallet_account(&mut rng, &admin_secret)
        .context("failed to create bridge admin account")?;
    let admin_account = set_nonce_to_one(admin_account);

    // 2. Create GER Manager
    let ger_secret = AuthSecretKey::new_falcon512_rpo_with_rng(&mut get_rpo_random_coin(&mut rng));
    let ger_account = build_wallet_account(&mut rng, &ger_secret)
        .context("failed to create GER manager account")?;
    let ger_account = set_nonce_to_one(ger_account);

    // 3. Create Bridge Account
    let bridge_seed: Word = rng.random::<[u64; 4]>().map(Felt::new).into();
    let bridge = create_bridge_account(bridge_seed, admin_account.id(), ger_account.id());

    // 4. Create Faucet
    let faucet_seed: Word = rng.random::<[u64; 4]>().map(Felt::new).into();
    let origin_token_address = EthAddressFormat::new(TEST_ORIGIN_TOKEN_ADDRESS);
    let faucet = create_agglayer_faucet(
        faucet_seed,
        "AGG",
        12,
        Felt::from(1_000_000_000u32),
        bridge.id(),
        &origin_token_address,
        0, // origin_network (mainnet)
        0, // scale
    );

    // Register the faucet in the bridge's faucet registry and set nonce to ONE
    let bridge = register_faucet_and_finalize(bridge, faucet.id())
        .context("failed to register faucet in bridge")?;
    let faucet = set_nonce_to_one(faucet);

    let admin_file = AccountFile::new(admin_account.clone(), vec![admin_secret]);
    let ger_file = AccountFile::new(ger_account.clone(), vec![ger_secret]);
    let bridge_file = AccountFile::new(bridge.clone(), vec![]);
    let faucet_file = AccountFile::new(faucet.clone(), vec![]);

    Ok(AgglayerGenesisAccounts {
        accounts: vec![admin_account, ger_account, bridge.clone(), faucet],
        account_files: vec![
            (BRIDGE_ADMIN_ACCOUNT_FILE, admin_file),
            (GER_MANAGER_ACCOUNT_FILE, ger_file),
            (BRIDGE_ACCOUNT_FILE, bridge_file),
            (AGGLAYER_FAUCET_ACCOUNT_FILE, faucet_file),
        ],
    })
}

/// Registers a faucet in the bridge's faucet registry and sets nonce to ONE.
///
/// Decomposes the account, mutates storage, and reassembles since `storage_mut()`
/// requires the `testing` feature on miden-protocol.
fn register_faucet_and_finalize(
    bridge: Account,
    faucet_id: miden_protocol::account::AccountId,
) -> Result<Account> {
    let (id, vault, mut storage, code, ..) = bridge.into_parts();

    let registry_key = StorageMapKey::new(faucet_registry_key(faucet_id));
    let registered_value = Word::new([ONE, ZERO, ZERO, ZERO]);

    storage
        .set_map_item(AggLayerBridge::faucet_registry_slot_name(), registry_key, registered_value)
        .context("failed to set faucet registry entry")?;

    Ok(Account::new_unchecked(id, vault, storage, code, ONE, None))
}

fn set_nonce_to_one(account: Account) -> Account {
    let (id, vault, storage, code, ..) = account.into_parts();
    Account::new_unchecked(id, vault, storage, code, ONE, None)
}

fn build_wallet_account(rng: &mut ChaCha20Rng, secret: &AuthSecretKey) -> Result<Account> {
    let seed: [u8; 32] = rng.random();

    let acc_component = AccountComponent::new(
        basic_wallet_library(),
        vec![],
        AccountComponentMetadata::new("miden::testing::basic_wallet").with_supports_all_types(),
    )
    .context("failed to create wallet component")?;

    let account = AccountBuilder::new(seed)
        .with_auth_component(AuthSingleSig::new(
            secret.public_key().to_commitment(),
            AuthScheme::Falcon512Rpo,
        ))
        .with_component(acc_component)
        .storage_mode(AccountStorageMode::Public)
        .build()
        .context("failed to build wallet account")?;

    Ok(account)
}
