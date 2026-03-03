-- Table for storing database migrations data.
-- Note: we can store values of different types in the same `value` field.
CREATE TABLE migrations (
    name  TEXT NOT NULL,
    value ANY,

    PRIMARY KEY (name),
    CONSTRAINT migration_name_is_not_empty CHECK (length(name) > 0)
) STRICT, WITHOUT ROWID;

-- Table for storing different settings in run-time, which need to persist over runs.
CREATE TABLE settings (
    name  TEXT NOT NULL,
    value BLOB NOT NULL,

    PRIMARY KEY (name),
    CONSTRAINT setting_name_is_not_empty CHECK (length(name) > 0)
) STRICT, WITHOUT ROWID;

-- Create account_code table
CREATE TABLE account_code (
    commitment TEXT NOT NULL,   -- commitment to the account code
    code BLOB NOT NULL,         -- serialized account code.
    PRIMARY KEY (commitment)
);

-- ── Account headers ──────────────────────────────────────────────────────

-- Latest account header: one row per account (current state).
CREATE TABLE latest_account_headers (
    id UNSIGNED BIG INT NOT NULL,            -- account ID
    account_commitment TEXT NOT NULL UNIQUE,  -- account state commitment
    code_commitment TEXT NOT NULL,            -- commitment to the account code
    storage_commitment TEXT NOT NULL,         -- commitment to the account storage
    vault_root TEXT NOT NULL,                 -- root of the account vault Merkle tree
    nonce BIGINT NOT NULL,                   -- account nonce
    account_seed BLOB NULL,                  -- seed used to generate the ID; NULL for non-new accounts
    locked BOOLEAN NOT NULL,                 -- whether the account is locked
    PRIMARY KEY (id),
    FOREIGN KEY (code_commitment) REFERENCES account_code(commitment)
);

-- Historical account headers: all state transitions (one row per nonce).
CREATE TABLE historical_account_headers (
    id UNSIGNED BIG INT NOT NULL,            -- account ID
    account_commitment TEXT NOT NULL UNIQUE,  -- account state commitment
    code_commitment TEXT NOT NULL,            -- commitment to the account code
    storage_commitment TEXT NOT NULL,         -- commitment to the account storage
    vault_root TEXT NOT NULL,                 -- root of the account vault Merkle tree
    nonce BIGINT NOT NULL,                   -- account nonce
    account_seed BLOB NULL,                  -- seed used to generate the ID; NULL for non-new accounts
    locked BOOLEAN NOT NULL,                 -- whether the account is locked
    PRIMARY KEY (account_commitment),
    FOREIGN KEY (code_commitment) REFERENCES account_code(commitment),

    CONSTRAINT check_seed_nonzero CHECK (NOT (nonce = 0 AND account_seed IS NULL))
);
CREATE INDEX idx_historical_account_headers_id_nonce ON historical_account_headers(id, nonce DESC);

-- ── Account storage (latest + historical) ────────────────────────────────

CREATE TABLE latest_account_storage (
    account_id TEXT NOT NULL,     -- account ID
    slot_name TEXT NOT NULL,      -- name of the storage slot
    slot_value TEXT NULL,         -- top-level value of the slot (for maps, contains the root)
    slot_type INTEGER NOT NULL,   -- type of the slot (0 = Value, 1 = Map)
    PRIMARY KEY (account_id, slot_name)
) WITHOUT ROWID;

CREATE TABLE historical_account_storage (
    account_id TEXT NOT NULL,     -- account ID
    nonce BIGINT NOT NULL,        -- nonce at which this slot was written
    slot_name TEXT NOT NULL,      -- name of the storage slot
    slot_value TEXT NULL,         -- top-level value of the slot (for maps, contains the root)
    slot_type INTEGER NOT NULL,   -- type of the slot (0 = Value, 1 = Map)
    PRIMARY KEY (account_id, nonce, slot_name)
) WITHOUT ROWID;

-- ── Storage map entries (latest + historical) ────────────────────────────

CREATE TABLE latest_storage_map_entries (
    account_id TEXT NOT NULL,   -- account ID
    slot_name TEXT NOT NULL,    -- name of the storage slot this entry belongs to
    key TEXT NOT NULL,          -- map entry key
    value TEXT NOT NULL,        -- map entry value
    PRIMARY KEY (account_id, slot_name, key)
) WITHOUT ROWID;

CREATE TABLE historical_storage_map_entries (
    account_id TEXT NOT NULL,   -- account ID
    nonce BIGINT NOT NULL,      -- nonce at which this entry was written
    slot_name TEXT NOT NULL,    -- name of the storage slot this entry belongs to
    key TEXT NOT NULL,          -- map entry key
    value TEXT NULL,            -- map entry value; NULL marks a removed entry (tombstone)
    PRIMARY KEY (account_id, nonce, slot_name, key)
) WITHOUT ROWID;

-- ── Account assets (latest + historical) ─────────────────────────────────

CREATE TABLE latest_account_assets (
    account_id TEXT NOT NULL,        -- account ID
    vault_key TEXT NOT NULL,         -- asset's vault key
    faucet_id_prefix TEXT NOT NULL,  -- prefix of the faucet ID, used to filter by faucet
    asset TEXT NOT NULL,             -- serialized asset value
    PRIMARY KEY (account_id, vault_key)
) WITHOUT ROWID;

CREATE TABLE historical_account_assets (
    account_id TEXT NOT NULL,        -- account ID
    nonce BIGINT NOT NULL,           -- nonce at which this asset was written
    vault_key TEXT NOT NULL,         -- asset's vault key
    faucet_id_prefix TEXT NOT NULL,  -- prefix of the faucet ID, used to filter by faucet
    asset TEXT NULL,                 -- serialized asset value; NULL marks a removed asset (tombstone)
    PRIMARY KEY (account_id, nonce, vault_key)
) WITHOUT ROWID;

-- ── Foreign account code ─────────────────────────────────────────────────

CREATE TABLE foreign_account_code(
    account_id TEXT NOT NULL,
    code_commitment TEXT NOT NULL,
    PRIMARY KEY (account_id),
    FOREIGN KEY (code_commitment) REFERENCES account_code(commitment)
);

-- ── Transactions ─────────────────────────────────────────────────────────

CREATE TABLE transactions (
    id TEXT NOT NULL,                                -- Transaction ID (commitment of various components)
    details BLOB NOT NULL,                           -- Serialized transaction details
    script_root TEXT,                                -- Transaction script root
    block_num UNSIGNED BIG INT,                      -- Block number for the block against which the transaction was executed.
    status_variant INT NOT NULL,                     -- Status variant identifier
    status BLOB NOT NULL,                            -- Serialized transaction status
    FOREIGN KEY (script_root) REFERENCES transaction_scripts(script_root),
    PRIMARY KEY (id)
) WITHOUT ROWID;
CREATE INDEX idx_transactions_uncommitted ON transactions(status_variant);


CREATE TABLE transaction_scripts (
    script_root TEXT NOT NULL,                       -- Transaction script root
    script BLOB,                                     -- serialized Transaction script

    PRIMARY KEY (script_root)
) WITHOUT ROWID;

-- ── Notes ────────────────────────────────────────────────────────────────

CREATE TABLE input_notes (
    note_id TEXT NOT NULL,                                  -- the note id
    assets BLOB NOT NULL,                                   -- the serialized list of assets
    serial_number BLOB NOT NULL,                            -- the serial number of the note
    inputs BLOB NOT NULL,                                   -- the serialized list of note inputs
    script_root TEXT NOT NULL,                              -- the script root of the note, used to join with the notes_scripts table
    nullifier TEXT NOT NULL,                                -- the nullifier of the note, used to query by nullifier
    state_discriminant UNSIGNED INT NOT NULL,               -- state discriminant of the note, used to query by state
    state BLOB NOT NULL,                                    -- serialized note state
    created_at UNSIGNED BIG INT NOT NULL,                   -- timestamp of the note creation/import

    PRIMARY KEY (note_id),
    FOREIGN KEY (script_root) REFERENCES notes_scripts(script_root)
) WITHOUT ROWID;
CREATE INDEX idx_input_notes_state ON input_notes(state_discriminant);
CREATE INDEX idx_input_notes_nullifier ON input_notes(nullifier);

CREATE TABLE output_notes (
    note_id TEXT NOT NULL,                                  -- the note id
    recipient_digest TEXT NOT NULL,                                -- the note recipient
    assets BLOB NOT NULL,                                   -- the serialized NoteAssets, including vault commitment and list of assets
    metadata BLOB NOT NULL,                                 -- serialized metadata
    nullifier TEXT NULL,
    expected_height UNSIGNED INT NOT NULL,                  -- the block height after which the note is expected to be created
-- TODO: normalize script data for output notes
--     script_commitment TEXT NULL,
    state_discriminant UNSIGNED INT NOT NULL,               -- state discriminant of the note, used to query by state
    state BLOB NOT NULL,                                    -- serialized note state

    PRIMARY KEY (note_id)
) WITHOUT ROWID;
CREATE INDEX idx_output_notes_state ON output_notes(state_discriminant);
CREATE INDEX idx_output_notes_nullifier ON output_notes(nullifier);

CREATE TABLE notes_scripts (
    script_root TEXT NOT NULL,                       -- Note script root
    serialized_note_script BLOB,                     -- NoteScript, serialized

    PRIMARY KEY (script_root)
);

-- ── State sync & tags ────────────────────────────────────────────────────

CREATE TABLE state_sync (
    block_num UNSIGNED BIG INT NOT NULL,    -- the block number of the most recent state sync
    PRIMARY KEY (block_num)
);

CREATE TABLE tags (
    tag BLOB NOT NULL,     -- the serialized tag
    source BLOB NOT NULL   -- the serialized tag source
);

-- insert initial row into state_sync table
INSERT OR IGNORE INTO state_sync (block_num)
SELECT 0
WHERE (
    SELECT COUNT(*) FROM state_sync
) = 0;

-- ── Block headers & partial blockchain ───────────────────────────────────

CREATE TABLE block_headers (
    block_num UNSIGNED BIG INT NOT NULL,  -- block number
    header BLOB NOT NULL,                 -- serialized block header
    partial_blockchain_peaks BLOB NOT NULL,        -- serialized peaks of the partial blockchain MMR at this block
    has_client_notes BOOL NOT NULL,       -- whether the block has notes relevant to the client
    PRIMARY KEY (block_num)
);
CREATE INDEX IF NOT EXISTS idx_block_headers_has_notes ON block_headers(block_num) WHERE has_client_notes = 1;

CREATE TABLE partial_blockchain_nodes (
    id UNSIGNED BIG INT NOT NULL,   -- in-order index of the internal MMR node
    node BLOB NOT NULL,             -- internal node value (commitment)
    PRIMARY KEY (id)
) WITHOUT ROWID;

-- ── Addresses ────────────────────────────────────────────────────────────

CREATE TABLE addresses (
    address BLOB NOT NULL,          -- the address
    account_id UNSIGNED BIG INT NOT NULL,   -- associated Account ID.

    PRIMARY KEY (address)
) WITHOUT ROWID;

CREATE INDEX idx_addresses_account_id ON addresses(account_id);
