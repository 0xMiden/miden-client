//! Shared SqlConnection-based account functions.

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use miden_client::account::{
    Account,
    AccountCode,
    AccountHeader,
    AccountId,
    AccountStorage,
    Address,
    PartialAccount,
    StorageMap,
    StorageSlot,
    StorageSlotContent,
    StorageSlotName,
    StorageSlotType,
};
use miden_client::asset::{Asset, AssetVault};
use miden_client::store::{
    AccountRecord,
    AccountRecordData,
    AccountStatus,
    AccountStorageFilter,
    StoreError,
};
use miden_client::sync::NoteTagRecord;
use miden_client::utils::Serializable;
use miden_client::{AccountError, Deserializable, Word};

use crate::sql_types::{SqlConnection, SqlParam, SqlRow};

// READER METHODS
// ================================================================================================

/// Get all tracked account IDs using [`SqlConnection`].
pub(crate) fn get_account_ids_shared(
    conn: &dyn SqlConnection,
) -> Result<Vec<AccountId>, StoreError> {
    let rows = conn.query_all("SELECT id FROM tracked_accounts", &[])?;
    rows.into_iter()
        .map(|row| {
            let id = row.get_text(0)?;
            Ok(AccountId::from_hex(id).expect("account id is valid"))
        })
        .collect()
}

/// Get account headers with status using [`SqlConnection`].
pub(crate) fn get_account_headers_shared(
    conn: &dyn SqlConnection,
) -> Result<Vec<(AccountHeader, AccountStatus)>, StoreError> {
    let rows = conn.query_all(
        "SELECT id, nonce, vault_root, storage_commitment, code_commitment, account_seed, locked \
         FROM accounts AS a \
         JOIN ( \
             SELECT id AS join_id, MAX(nonce) AS max_nonce \
             FROM accounts \
             GROUP BY id \
         ) AS latest \
         ON a.id = latest.join_id \
         AND a.nonce = latest.max_nonce \
         ORDER BY a.id",
        &[],
    )?;
    rows.into_iter().map(parse_account_header_row).collect()
}

/// Get a single account header by ID using [`SqlConnection`].
pub(crate) fn get_account_header_shared(
    conn: &dyn SqlConnection,
    account_id: AccountId,
) -> Result<Option<(AccountHeader, AccountStatus)>, StoreError> {
    let row = conn.query_one(
        "SELECT id, nonce, vault_root, storage_commitment, code_commitment, account_seed, locked \
         FROM accounts WHERE id = ? ORDER BY nonce DESC LIMIT 1",
        &[SqlParam::Text(account_id.to_hex())],
    )?;
    row.map(parse_account_header_row).transpose()
}

/// Get account header by commitment using [`SqlConnection`].
pub(crate) fn get_account_header_by_commitment_shared(
    conn: &dyn SqlConnection,
    account_commitment: Word,
) -> Result<Option<AccountHeader>, StoreError> {
    let row = conn.query_one(
        "SELECT id, nonce, vault_root, storage_commitment, code_commitment, account_seed, locked \
         FROM accounts WHERE account_commitment = ?",
        &[SqlParam::Text(account_commitment.to_string())],
    )?;
    row.map(|r| parse_account_header_row(r).map(|(header, _)| header)).transpose()
}

/// Get a complete account record using [`SqlConnection`].
pub(crate) fn get_account_shared(
    conn: &dyn SqlConnection,
    account_id: AccountId,
) -> Result<Option<AccountRecord>, StoreError> {
    let Some((header, status)) = get_account_header_shared(conn, account_id)? else {
        return Ok(None);
    };

    let assets = query_vault_assets_shared(conn, header.vault_root())?;
    let vault = AssetVault::new(&assets)?;

    let slots = query_storage_slots_shared(conn, header.storage_commitment())?
        .into_values()
        .collect();
    let storage = AccountStorage::new(slots)?;

    let Some(account_code) = query_account_code_shared(conn, header.code_commitment())? else {
        return Ok(None);
    };

    let account = miden_client::account::Account::new_unchecked(
        header.id(),
        vault,
        storage,
        account_code,
        header.nonce(),
        status.seed().copied(),
    );

    let account_data = AccountRecordData::Full(account);
    Ok(Some(AccountRecord::new(account_data, status)))
}

/// Get account code by account ID using [`SqlConnection`].
pub(crate) fn get_account_code_by_id_shared(
    conn: &dyn SqlConnection,
    account_id: AccountId,
) -> Result<Option<AccountCode>, StoreError> {
    let Some((header, _)) = get_account_header_shared(conn, account_id)? else {
        return Ok(None);
    };
    query_account_code_shared(conn, header.code_commitment())
}

/// Get foreign account code using [`SqlConnection`] (no `rarray`).
pub(crate) fn get_foreign_account_code_shared(
    conn: &dyn SqlConnection,
    account_ids: &[AccountId],
) -> Result<BTreeMap<AccountId, AccountCode>, StoreError> {
    if account_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let placeholders: String = account_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let sql = format!(
        "SELECT account_id, code \
         FROM foreign_account_code \
         JOIN account_code ON foreign_account_code.code_commitment = account_code.commitment \
         WHERE account_id IN ({placeholders})"
    );
    let params: Vec<SqlParam> = account_ids.iter().map(|id| SqlParam::Text(id.to_hex())).collect();

    let rows = conn.query_all(&sql, &params)?;
    rows.into_iter()
        .map(|row| {
            let id_str = row.get_text(0)?;
            let code_bytes = row.get_blob(1)?;
            let id = AccountId::from_hex(id_str).map_err(|err| {
                StoreError::AccountError(AccountError::FinalAccountHeaderIdParsingFailed(err))
            })?;
            let code = AccountCode::from_bytes(code_bytes).map_err(StoreError::AccountError)?;
            Ok((id, code))
        })
        .collect()
}

/// Get account vault using [`SqlConnection`].
pub(crate) fn get_account_vault_shared(
    conn: &dyn SqlConnection,
    account_id: AccountId,
) -> Result<AssetVault, StoreError> {
    let rows = conn.query_all(
        "SELECT asset FROM account_assets \
         WHERE root = (SELECT vault_root FROM accounts WHERE id = ? ORDER BY nonce DESC LIMIT 1)",
        &[SqlParam::Text(account_id.to_hex())],
    )?;
    let assets: Vec<Asset> = rows
        .into_iter()
        .map(|row| {
            let word = Word::try_from(row.get_text(0)?)?;
            Ok(Asset::try_from(word)?)
        })
        .collect::<Result<Vec<_>, StoreError>>()?;

    Ok(AssetVault::new(&assets)?)
}

/// Get account storage using [`SqlConnection`].
pub(crate) fn get_account_storage_shared(
    conn: &dyn SqlConnection,
    account_id: AccountId,
    filter: &AccountStorageFilter,
) -> Result<AccountStorage, StoreError> {
    let (where_clause, params) = match filter {
        AccountStorageFilter::All => (
            "commitment = (SELECT storage_commitment FROM accounts WHERE id = ? ORDER BY nonce DESC LIMIT 1)".to_string(),
            vec![SqlParam::Text(account_id.to_hex())],
        ),
        AccountStorageFilter::Root(root) => (
            "commitment = (SELECT storage_commitment FROM accounts WHERE id = ? ORDER BY nonce DESC LIMIT 1) AND slot_value = ?".to_string(),
            vec![SqlParam::Text(account_id.to_hex()), SqlParam::Text(root.to_hex())],
        ),
        AccountStorageFilter::SlotName(slot_name) => (
            "commitment = (SELECT storage_commitment FROM accounts WHERE id = ? ORDER BY nonce DESC LIMIT 1) AND slot_name = ?".to_string(),
            vec![SqlParam::Text(account_id.to_hex()), SqlParam::Text(slot_name.to_string())],
        ),
    };

    let slots = query_storage_slots_with_where_shared(conn, &where_clause, &params)?
        .into_values()
        .collect();
    Ok(AccountStorage::new(slots)?)
}

/// Get account addresses using [`SqlConnection`].
pub(crate) fn get_account_addresses_shared(
    conn: &dyn SqlConnection,
    account_id: AccountId,
) -> Result<Vec<Address>, StoreError> {
    let rows = conn.query_all(
        "SELECT address FROM addresses WHERE account_id = ?",
        &[SqlParam::Text(account_id.to_hex())],
    )?;
    rows.into_iter()
        .map(|row| {
            let serialized = row.get_blob(0)?;
            Ok(Address::read_from_bytes(serialized)?)
        })
        .collect()
}

// WRITER METHODS
// ================================================================================================

/// Upsert foreign account code using [`SqlConnection`].
pub(crate) fn upsert_foreign_account_code_shared(
    conn: &dyn SqlConnection,
    account_id: AccountId,
    code: &AccountCode,
) -> Result<(), StoreError> {
    // Insert account code
    insert_account_code_shared(conn, code)?;

    // Insert foreign account code mapping
    const QUERY: &str = insert_sql!(foreign_account_code { account_id, code_commitment } | REPLACE);
    conn.execute(
        QUERY,
        &[
            SqlParam::Text(account_id.to_hex()),
            SqlParam::Text(code.commitment().to_string()),
        ],
    )?;

    Ok(())
}

/// Insert account code using [`SqlConnection`].
pub(crate) fn insert_account_code_shared(
    conn: &dyn SqlConnection,
    code: &AccountCode,
) -> Result<(), StoreError> {
    const QUERY: &str = insert_sql!(account_code { commitment, code } | IGNORE);
    conn.execute(
        QUERY,
        &[SqlParam::Text(code.commitment().to_hex()), SqlParam::Blob(code.to_bytes())],
    )?;
    Ok(())
}

/// Insert an address using [`SqlConnection`].
pub(crate) fn insert_address_shared(
    conn: &dyn SqlConnection,
    address: &Address,
    account_id: AccountId,
) -> Result<(), StoreError> {
    let derived_note_tag = address.to_note_tag();
    let note_tag_record = NoteTagRecord::with_account_source(derived_note_tag, account_id);

    crate::sync::insert_note_tag_shared(conn, &note_tag_record)?;

    const QUERY: &str = insert_sql!(addresses { address, account_id } | REPLACE);
    conn.execute(
        QUERY,
        &[SqlParam::Blob(address.to_bytes()), SqlParam::Text(account_id.to_hex())],
    )?;
    Ok(())
}

/// Remove an address using [`SqlConnection`].
pub(crate) fn remove_address_shared(
    conn: &dyn SqlConnection,
    address: &Address,
    account_id: AccountId,
) -> Result<(), StoreError> {
    let derived_note_tag = address.to_note_tag();
    let note_tag_record = NoteTagRecord::with_account_source(derived_note_tag, account_id);

    crate::sync::remove_note_tag_shared(conn, &note_tag_record)?;

    conn.execute("DELETE FROM addresses WHERE address = ?", &[SqlParam::Blob(address.to_bytes())])?;
    Ok(())
}

/// Insert storage slots and their map entries using [`SqlConnection`].
#[allow(dead_code)]
pub(crate) fn insert_storage_slots_shared<'a>(
    conn: &dyn SqlConnection,
    commitment: Word,
    account_storage: impl Iterator<Item = &'a StorageSlot>,
) -> Result<(), StoreError> {
    let commitment_hex = commitment.to_hex();
    for slot in account_storage {
        conn.execute(
            insert_sql!(
                account_storage {
                    commitment,
                    slot_name,
                    slot_value,
                    slot_type
                } | REPLACE
            ),
            &[
                SqlParam::Text(commitment_hex.clone()),
                SqlParam::Text(slot.name().to_string()),
                SqlParam::Text(slot.value().to_hex()),
                SqlParam::from(slot.slot_type() as u8),
            ],
        )?;

        if let StorageSlotContent::Map(map) = slot.content() {
            let root_hex = map.root().to_hex();
            for (key, value) in map.entries() {
                conn.execute(
                    insert_sql!(storage_map_entries { root, key, value } | REPLACE),
                    &[
                        SqlParam::Text(root_hex.clone()),
                        SqlParam::Text(key.to_hex()),
                        SqlParam::Text(value.to_hex()),
                    ],
                )?;
            }
        }
    }
    Ok(())
}

/// Insert vault assets using [`SqlConnection`].
#[allow(dead_code)]
pub(crate) fn insert_assets_shared(
    conn: &dyn SqlConnection,
    root: Word,
    assets: impl Iterator<Item = Asset>,
) -> Result<(), StoreError> {
    let root_hex = root.to_hex();
    for asset in assets {
        let vault_key_word: Word = asset.vault_key().into();
        conn.execute(
            insert_sql!(account_assets { root, vault_key, faucet_id_prefix, asset } | REPLACE),
            &[
                SqlParam::Text(root_hex.clone()),
                SqlParam::Text(vault_key_word.to_hex()),
                SqlParam::Text(asset.faucet_id_prefix().to_hex()),
                SqlParam::Text(Word::from(asset).to_hex()),
            ],
        )?;
    }
    Ok(())
}

/// Insert an account header row using [`SqlConnection`].
#[allow(dead_code)]
pub(crate) fn insert_account_header_shared(
    conn: &dyn SqlConnection,
    account: &AccountHeader,
    account_seed: Option<Word>,
) -> Result<(), StoreError> {
    let id = account.id().to_hex();
    let code_commitment = account.code_commitment().to_string();
    let storage_commitment = account.storage_commitment().to_string();
    let vault_root = account.vault_root().to_string();
    let nonce = account.nonce().as_int();
    let commitment = account.commitment().to_string();
    let account_seed_bytes = account_seed.map(|seed| seed.to_bytes());

    conn.execute(
        insert_sql!(
            accounts {
                id,
                code_commitment,
                storage_commitment,
                vault_root,
                nonce,
                account_seed,
                account_commitment,
                locked
            } | REPLACE
        ),
        &[
            SqlParam::Text(id),
            SqlParam::Text(code_commitment),
            SqlParam::Text(storage_commitment),
            SqlParam::Text(vault_root),
            SqlParam::from(nonce),
            SqlParam::from(account_seed_bytes),
            SqlParam::Text(commitment),
            SqlParam::from(false),
        ],
    )?;

    insert_tracked_account_id_shared(conn, account.id())?;
    Ok(())
}

/// Insert a tracked account ID using [`SqlConnection`].
#[allow(dead_code)]
pub(crate) fn insert_tracked_account_id_shared(
    conn: &dyn SqlConnection,
    account_id: AccountId,
) -> Result<(), StoreError> {
    conn.execute(
        insert_sql!(tracked_accounts { id } | IGNORE),
        &[SqlParam::Text(account_id.to_hex())],
    )?;
    Ok(())
}

/// Insert a complete account (code, storage, vault, header, address) using [`SqlConnection`].
#[allow(dead_code)]
pub(crate) fn insert_account_shared(
    conn: &dyn SqlConnection,
    account: &Account,
    initial_address: &Address,
) -> Result<(), StoreError> {
    insert_account_code_shared(conn, account.code())?;
    insert_storage_slots_shared(
        conn,
        account.storage().to_commitment(),
        account.storage().slots().iter(),
    )?;
    insert_assets_shared(conn, account.vault().root(), account.vault().assets())?;
    insert_account_header_shared(conn, &account.into(), account.seed())?;
    insert_address_shared(conn, initial_address, account.id())?;
    Ok(())
}

/// Update account state by full replacement (no SMT/delta) using [`SqlConnection`].
///
/// Inserts the new code, storage, vault, and header. Does NOT update SMT forest.
#[allow(dead_code)]
pub(crate) fn update_account_state_shared(
    conn: &dyn SqlConnection,
    account: &Account,
) -> Result<(), StoreError> {
    insert_account_code_shared(conn, account.code())?;
    insert_storage_slots_shared(
        conn,
        account.storage().to_commitment(),
        account.storage().slots().iter(),
    )?;
    insert_assets_shared(conn, account.vault().root(), account.vault().assets())?;
    insert_account_header_shared(conn, &account.into(), None)?;
    Ok(())
}

/// Check account exists, then update its state using [`SqlConnection`].
#[allow(dead_code)]
pub(crate) fn update_account_shared(
    conn: &dyn SqlConnection,
    account: &Account,
) -> Result<(), StoreError> {
    let exists = conn.query_one(
        "SELECT id FROM accounts WHERE id = ? LIMIT 1",
        &[SqlParam::Text(account.id().to_hex())],
    )?;
    if exists.is_none() {
        return Err(StoreError::AccountDataNotFound(account.id()));
    }

    update_account_state_shared(conn, account)?;
    Ok(())
}

/// Delete account states by commitment using [`SqlConnection`] (no SMT cleanup).
#[allow(dead_code)]
pub(crate) fn undo_account_states_shared(
    conn: &dyn SqlConnection,
    account_commitments: &[Word],
) -> Result<(), StoreError> {
    if account_commitments.is_empty() {
        return Ok(());
    }

    let placeholders: String =
        account_commitments.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let sql = format!("DELETE FROM accounts WHERE account_commitment IN ({placeholders})");
    let params: Vec<SqlParam> =
        account_commitments.iter().map(|h| SqlParam::Text(h.to_string())).collect();

    conn.execute(&sql, &params)?;
    Ok(())
}

/// Lock an account on unexpected commitment using [`SqlConnection`].
#[allow(dead_code)]
pub(crate) fn lock_account_on_unexpected_commitment_shared(
    conn: &dyn SqlConnection,
    account_id: &AccountId,
    mismatched_digest: &Word,
) -> Result<(), StoreError> {
    conn.execute(
        "UPDATE accounts SET locked = true WHERE id = ?1 AND NOT EXISTS \
         (SELECT 1 FROM accounts WHERE id = ?1 AND account_commitment = ?2)",
        &[
            SqlParam::Text(account_id.to_hex()),
            SqlParam::Text(mismatched_digest.to_string()),
        ],
    )?;
    Ok(())
}

/// Get a minimal partial account record using [`SqlConnection`] (no SMT witnesses).
///
/// This builds a full account from the DB and converts it to a partial representation.
#[allow(dead_code)]
pub(crate) fn get_minimal_partial_account_shared(
    conn: &dyn SqlConnection,
    account_id: AccountId,
) -> Result<Option<AccountRecord>, StoreError> {
    let Some((header, status)) = get_account_header_shared(conn, account_id)? else {
        return Ok(None);
    };

    let assets = query_vault_assets_shared(conn, header.vault_root())?;
    let vault = AssetVault::new(&assets)?;

    let slots = query_storage_slots_shared(conn, header.storage_commitment())?
        .into_values()
        .collect();
    let storage = AccountStorage::new(slots)?;

    let Some(account_code) = query_account_code_shared(conn, header.code_commitment())? else {
        return Ok(None);
    };

    let account = Account::new_unchecked(
        header.id(),
        vault,
        storage,
        account_code,
        header.nonce(),
        status.seed().copied(),
    );

    let partial_account = PartialAccount::from(&account);
    let account_data = AccountRecordData::Partial(partial_account);
    Ok(Some(AccountRecord::new(account_data, status)))
}

// HELPERS
// ================================================================================================

/// Parse an account header from a [`SqlRow`].
#[allow(clippy::needless_pass_by_value)]
fn parse_account_header_row(row: SqlRow) -> Result<(AccountHeader, AccountStatus), StoreError> {
    let id = row.get_text(0)?;
    let nonce = row.get_u64(1)?;
    let vault_root = row.get_text(2)?;
    let storage_commitment = row.get_text(3)?;
    let code_commitment = row.get_text(4)?;
    let account_seed = row.get_optional_blob(5)?;
    let locked = row.get_bool(6)?;

    let account_seed = account_seed.map(Word::read_from_bytes).transpose()?;

    let status = match (account_seed, locked) {
        (seed, true) => AccountStatus::Locked { seed },
        (Some(seed), _) => AccountStatus::New { seed },
        _ => AccountStatus::Tracked,
    };

    Ok((
        AccountHeader::new(
            AccountId::from_hex(id).expect("Conversion from stored AccountID should not panic"),
            miden_client::Felt::new(nonce),
            Word::try_from(vault_root)?,
            Word::try_from(storage_commitment)?,
            Word::try_from(code_commitment)?,
        ),
        status,
    ))
}

/// Query account code by commitment using [`SqlConnection`].
fn query_account_code_shared(
    conn: &dyn SqlConnection,
    commitment: Word,
) -> Result<Option<AccountCode>, StoreError> {
    let row = conn.query_one(
        "SELECT code FROM account_code WHERE commitment = ?",
        &[SqlParam::Text(commitment.to_hex())],
    )?;
    match row {
        Some(row) => {
            let bytes = row.get_blob(0)?;
            Ok(Some(AccountCode::from_bytes(bytes)?))
        },
        None => Ok(None),
    }
}

/// Query vault assets by vault root using [`SqlConnection`].
fn query_vault_assets_shared(
    conn: &dyn SqlConnection,
    vault_root: Word,
) -> Result<Vec<Asset>, StoreError> {
    let rows = conn.query_all(
        "SELECT asset FROM account_assets WHERE root = ?",
        &[SqlParam::Text(vault_root.to_hex())],
    )?;
    rows.into_iter()
        .map(|row| {
            let word = Word::try_from(row.get_text(0)?)?;
            Ok(Asset::try_from(word)?)
        })
        .collect()
}

/// Query storage slots by storage commitment using [`SqlConnection`].
fn query_storage_slots_shared(
    conn: &dyn SqlConnection,
    storage_commitment: Word,
) -> Result<BTreeMap<StorageSlotName, StorageSlot>, StoreError> {
    query_storage_slots_with_where_shared(
        conn,
        "commitment = ?",
        &[SqlParam::Text(storage_commitment.to_hex())],
    )
}

/// Query storage slots with a custom WHERE clause using [`SqlConnection`].
fn query_storage_slots_with_where_shared(
    conn: &dyn SqlConnection,
    where_clause: &str,
    params: &[SqlParam],
) -> Result<BTreeMap<StorageSlotName, StorageSlot>, StoreError> {
    let sql = format!(
        "SELECT slot_name, slot_value, slot_type FROM account_storage WHERE {where_clause}"
    );
    let rows = conn.query_all(&sql, params)?;

    let mut storage_values = Vec::new();
    let mut possible_roots = Vec::new();

    for row in rows {
        let slot_name = row.get_text(0)?.to_string();
        let value = row.get_text(1)?.to_string();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let slot_type: u8 = row.get_i64(2)? as u8;

        let slot_name = StorageSlotName::new(slot_name)
            .map_err(|err| StoreError::ParsingError(err.to_string()))?;
        let slot_type = StorageSlotType::try_from(slot_type)
            .map_err(|e| StoreError::ParsingError(e.to_string()))?;
        let word = Word::try_from(value.as_str())?;

        possible_roots.push(word);
        storage_values.push((slot_name, word, slot_type));
    }

    // Fetch storage maps for all possible roots
    let mut storage_maps = query_storage_maps_shared(conn, &possible_roots)?;

    Ok(storage_values
        .into_iter()
        .map(|(slot_name, value, slot_type)| {
            let key = slot_name.clone();
            let slot = match slot_type {
                StorageSlotType::Value => StorageSlot::with_value(slot_name, value),
                StorageSlotType::Map => StorageSlot::with_map(
                    slot_name,
                    storage_maps.remove(&value).unwrap_or(StorageMap::new()),
                ),
            };
            (key, slot)
        })
        .collect())
}

/// Query storage maps by roots using [`SqlConnection`] (no `rarray`).
fn query_storage_maps_shared(
    conn: &dyn SqlConnection,
    roots: &[Word],
) -> Result<BTreeMap<Word, StorageMap>, StoreError> {
    if roots.is_empty() {
        return Ok(BTreeMap::new());
    }

    let placeholders: String = roots.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let sql =
        format!("SELECT root, key, value FROM storage_map_entries WHERE root IN ({placeholders})");
    let params: Vec<SqlParam> = roots.iter().map(|r| SqlParam::Text(r.to_hex())).collect();

    let rows = conn.query_all(&sql, &params)?;
    let mut maps = BTreeMap::new();
    for row in rows {
        let root = Word::try_from(row.get_text(0)?)?;
        let key = Word::try_from(row.get_text(1)?)?;
        let value = Word::try_from(row.get_text(2)?)?;
        let map = maps.entry(root).or_insert_with(StorageMap::new);
        map.insert(key, value)?;
    }

    Ok(maps)
}
