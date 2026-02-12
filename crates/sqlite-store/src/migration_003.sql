-- Migration 003: Split accounts table into latest/historical (issue #1768)
--
-- Creates latest_account_headers (one row per account, PK on id) and renames
-- accounts to historical_account_headers.  Drops tracked_accounts since
-- latest_account_headers replaces it.

-- ── Create latest_account_headers ────────────────────────────────────────

CREATE TABLE latest_account_headers (
    id UNSIGNED BIG INT NOT NULL,
    account_commitment TEXT NOT NULL UNIQUE,
    code_commitment TEXT NOT NULL,
    storage_commitment TEXT NOT NULL,
    vault_root TEXT NOT NULL,
    nonce BIGINT NOT NULL,
    account_seed BLOB NULL,
    locked BOOLEAN NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (code_commitment) REFERENCES account_code(commitment)
);

-- ── Populate from accounts using MAX(nonce) per id ───────────────────────

INSERT INTO latest_account_headers (id, account_commitment, code_commitment, storage_commitment, vault_root, nonce, account_seed, locked)
SELECT a.id, a.account_commitment, a.code_commitment, a.storage_commitment, a.vault_root, a.nonce, a.account_seed, a.locked
FROM accounts a
JOIN (
    SELECT id, MAX(nonce) AS max_nonce FROM accounts GROUP BY id
) latest ON a.id = latest.id AND a.nonce = latest.max_nonce;

-- ── Rename accounts → historical_account_headers ─────────────────────────

ALTER TABLE accounts RENAME TO historical_account_headers;

-- ── Drop tracked_accounts (replaced by latest_account_headers) ───────────

DROP TABLE tracked_accounts;
