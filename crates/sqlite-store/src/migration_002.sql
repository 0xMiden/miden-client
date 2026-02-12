-- Migration 002: Restructure account state storage model (issue #1768)
--
-- Re-keys account_storage, storage_map_entries, and account_assets by
-- (account_id, nonce) instead of Merkle roots. Splits each into a "latest"
-- table (full current state) and a "historical" table (entries per nonce).

-- ── Latest tables (full current state per account) ──────────────────────

CREATE TABLE latest_account_storage (
    account_id TEXT NOT NULL,
    slot_name TEXT NOT NULL,
    slot_value TEXT NULL,
    slot_type INTEGER NOT NULL,
    PRIMARY KEY (account_id, slot_name)
) WITHOUT ROWID;

CREATE TABLE latest_storage_map_entries (
    account_id TEXT NOT NULL,
    slot_name TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (account_id, slot_name, key)
) WITHOUT ROWID;

CREATE TABLE latest_account_assets (
    account_id TEXT NOT NULL,
    vault_key TEXT NOT NULL,
    faucet_id_prefix TEXT NOT NULL,
    asset TEXT NOT NULL,
    PRIMARY KEY (account_id, vault_key)
) WITHOUT ROWID;

-- ── Historical tables (changed entries per nonce) ───────────────────────

CREATE TABLE historical_account_storage (
    account_id TEXT NOT NULL,
    nonce BIGINT NOT NULL,
    slot_name TEXT NOT NULL,
    slot_value TEXT NULL,
    slot_type INTEGER NOT NULL,
    PRIMARY KEY (account_id, nonce, slot_name)
) WITHOUT ROWID;

CREATE TABLE historical_storage_map_entries (
    account_id TEXT NOT NULL,
    nonce BIGINT NOT NULL,
    slot_name TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NULL,
    PRIMARY KEY (account_id, nonce, slot_name, key)
) WITHOUT ROWID;

CREATE TABLE historical_account_assets (
    account_id TEXT NOT NULL,
    nonce BIGINT NOT NULL,
    vault_key TEXT NOT NULL,
    faucet_id_prefix TEXT NOT NULL,
    asset TEXT NULL,
    PRIMARY KEY (account_id, nonce, vault_key)
) WITHOUT ROWID;

-- ── Populate latest tables from current state (MAX nonce per account) ───

INSERT INTO latest_account_storage (account_id, slot_name, slot_value, slot_type)
SELECT a.id, s.slot_name, s.slot_value, s.slot_type
FROM account_storage s
JOIN accounts a ON a.storage_commitment = s.commitment
JOIN (
    SELECT id, MAX(nonce) AS max_nonce FROM accounts GROUP BY id
) latest ON a.id = latest.id AND a.nonce = latest.max_nonce;

INSERT INTO latest_storage_map_entries (account_id, slot_name, key, value)
SELECT a.id, s.slot_name, m.key, m.value
FROM storage_map_entries m
JOIN account_storage s ON s.slot_value = m.root AND s.slot_type = 1
JOIN accounts a ON a.storage_commitment = s.commitment
JOIN (
    SELECT id, MAX(nonce) AS max_nonce FROM accounts GROUP BY id
) latest ON a.id = latest.id AND a.nonce = latest.max_nonce;

INSERT INTO latest_account_assets (account_id, vault_key, faucet_id_prefix, asset)
SELECT a.id, v.vault_key, v.faucet_id_prefix, v.asset
FROM account_assets v
JOIN accounts a ON a.vault_root = v.root
JOIN (
    SELECT id, MAX(nonce) AS max_nonce FROM accounts GROUP BY id
) latest ON a.id = latest.id AND a.nonce = latest.max_nonce;

-- ── Populate historical tables from ALL account states ──────────────────

INSERT INTO historical_account_storage (account_id, nonce, slot_name, slot_value, slot_type)
SELECT a.id, a.nonce, s.slot_name, s.slot_value, s.slot_type
FROM account_storage s
JOIN accounts a ON a.storage_commitment = s.commitment;

INSERT INTO historical_storage_map_entries (account_id, nonce, slot_name, key, value)
SELECT a.id, a.nonce, s.slot_name, m.key, m.value
FROM storage_map_entries m
JOIN account_storage s ON s.slot_value = m.root AND s.slot_type = 1
JOIN accounts a ON a.storage_commitment = s.commitment;

INSERT INTO historical_account_assets (account_id, nonce, vault_key, faucet_id_prefix, asset)
SELECT a.id, a.nonce, v.vault_key, v.faucet_id_prefix, v.asset
FROM account_assets v
JOIN accounts a ON a.vault_root = v.root;

-- ── Drop old tables and their indexes ───────────────────────────────────

DROP INDEX IF EXISTS idx_account_storage_commitment;
DROP INDEX IF EXISTS idx_storage_map_entries_root;
DROP INDEX IF EXISTS idx_account_assets_root;
DROP INDEX IF EXISTS idx_account_assets_root_faucet_prefix;

DROP TABLE account_storage;
DROP TABLE storage_map_entries;
DROP TABLE account_assets;
