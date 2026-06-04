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
    watched BOOLEAN NOT NULL DEFAULT FALSE, -- Whether the account is tracked in watch mode
    PRIMARY KEY (id),
    FOREIGN KEY (code_commitment) REFERENCES account_code(commitment)
);

-- Historical account headers: stores old headers that were replaced by newer states.
-- Each row represents a previous account state that was superseded at replaced_at_nonce.
CREATE TABLE historical_account_headers (
    id UNSIGNED BIG INT NOT NULL,            -- account ID
    account_commitment TEXT NOT NULL UNIQUE,  -- commitment of this old state
    code_commitment TEXT NOT NULL,            -- commitment to the old account code
    storage_commitment TEXT NOT NULL,         -- commitment to the old account storage
    vault_root TEXT NOT NULL,                 -- root of the old account vault Merkle tree
    nonce BIGINT NOT NULL,                   -- nonce of this old state
    account_seed BLOB NULL,                  -- seed used to generate the ID; NULL for non-new accounts
    locked BOOLEAN NOT NULL,                 -- whether the account was locked
    replaced_at_nonce BIGINT NOT NULL,       -- nonce of the new state that replaced this one
    PRIMARY KEY (account_commitment),
    FOREIGN KEY (code_commitment) REFERENCES account_code(commitment),

    CONSTRAINT check_seed_nonzero CHECK (NOT (nonce = 0 AND account_seed IS NULL))
);
CREATE INDEX idx_historical_account_headers_id_replaced_at ON historical_account_headers(id, replaced_at_nonce DESC);

-- ── Account storage (latest + historical) ────────────────────────────────

CREATE TABLE latest_account_storage (
    account_id TEXT NOT NULL,     -- account ID
    slot_name TEXT NOT NULL,      -- name of the storage slot
    slot_value TEXT NULL,         -- top-level value of the slot (for maps, contains the root)
    slot_type INTEGER NOT NULL,   -- type of the slot (0 = Value, 1 = Map)
    PRIMARY KEY (account_id, slot_name)
) WITHOUT ROWID;

-- Historical account storage: stores old slot values that were replaced.
-- NULL old_slot_value means the slot didn't exist before (was created at replaced_at_nonce).
CREATE TABLE historical_account_storage (
    account_id TEXT NOT NULL,           -- account ID
    replaced_at_nonce BIGINT NOT NULL,  -- nonce at which this old value was replaced
    slot_name TEXT NOT NULL,            -- name of the storage slot
    old_slot_value TEXT NULL,           -- old top-level value (NULL = slot was new)
    slot_type INTEGER NOT NULL,         -- type of the slot (0 = Value, 1 = Map)
    PRIMARY KEY (account_id, replaced_at_nonce, slot_name)
) WITHOUT ROWID;

-- ── Storage map entries (latest + historical) ────────────────────────────

CREATE TABLE latest_storage_map_entries (
    account_id TEXT NOT NULL,   -- account ID
    slot_name TEXT NOT NULL,    -- name of the storage slot this entry belongs to
    key TEXT NOT NULL,          -- map entry key
    value TEXT NOT NULL,        -- map entry value
    PRIMARY KEY (account_id, slot_name, key)
) WITHOUT ROWID;

-- Historical storage map entries: stores old map entry values that were replaced.
-- NULL old_value means the entry didn't exist before (was created at replaced_at_nonce).
CREATE TABLE historical_storage_map_entries (
    account_id TEXT NOT NULL,           -- account ID
    replaced_at_nonce BIGINT NOT NULL,  -- nonce at which this old entry was replaced
    slot_name TEXT NOT NULL,            -- name of the storage slot this entry belongs to
    key TEXT NOT NULL,                  -- map entry key
    old_value TEXT NULL,                -- old map entry value (NULL = entry was new)
    PRIMARY KEY (account_id, replaced_at_nonce, slot_name, key)
) WITHOUT ROWID;

-- ── Account assets (latest + historical) ─────────────────────────────────

CREATE TABLE latest_account_assets (
    account_id TEXT NOT NULL,        -- account ID
    vault_key TEXT NOT NULL,         -- asset's vault key
    asset TEXT NOT NULL,             -- serialized asset value
    PRIMARY KEY (account_id, vault_key)
) WITHOUT ROWID;

-- Historical account assets: stores old assets that were replaced.
-- NULL old_asset means the asset didn't exist before (was created at replaced_at_nonce).
CREATE TABLE historical_account_assets (
    account_id TEXT NOT NULL,           -- account ID
    replaced_at_nonce BIGINT NOT NULL,  -- nonce at which this old asset was replaced
    vault_key TEXT NOT NULL,            -- asset's vault key
    old_asset TEXT NULL,                -- old serialized asset value (NULL = asset was new)
    PRIMARY KEY (account_id, replaced_at_nonce, vault_key)
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
    details_commitment TEXT NOT NULL,                       -- commitment to the note details (recipient + assets); stable across the note's lifecycle and independent of metadata
    note_id TEXT NULL,                                      -- the full note id (hash(details_commitment, metadata_commitment)); NULL until metadata is known
    assets BLOB NOT NULL,                                   -- the serialized list of assets
    attachments BLOB NOT NULL,                              -- the serialized NoteAttachments
    serial_number BLOB NOT NULL,                            -- the serial number of the note
    inputs BLOB NOT NULL,                                   -- the serialized list of note inputs
    script_root TEXT NOT NULL,                              -- the script root of the note, used to join with the notes_scripts table
    nullifier TEXT NULL,                                    -- the nullifier of the note, used to query by nullifier; NULL until metadata is known
    state_discriminant UNSIGNED INT NOT NULL,               -- state discriminant of the note, used to query by state
    state BLOB NOT NULL,                                    -- serialized note state
    created_at UNSIGNED BIG INT NOT NULL,                   -- timestamp of the note creation/import
    consumed_block_height INTEGER NULL,                     -- block height at which the note was consumed; NULL for non-consumed notes
    consumed_tx_order INTEGER NULL,                         -- per-account position of the consuming tx in the account's execution chain within the block; NULL for external consumption or non-consumed notes
    consumer_account_id TEXT NULL,                          -- account ID that consumed this note; NULL for non-consumed or externally consumed notes

    PRIMARY KEY (details_commitment),
    FOREIGN KEY (script_root) REFERENCES notes_scripts(script_root)
) WITHOUT ROWID;
CREATE INDEX idx_input_notes_state ON input_notes(state_discriminant);
CREATE INDEX idx_input_notes_nullifier ON input_notes(nullifier);
CREATE INDEX idx_input_notes_note_id ON input_notes(note_id);
CREATE INDEX idx_input_notes_consumption ON input_notes(consumed_block_height, consumed_tx_order);

CREATE TABLE output_notes (
    details_commitment TEXT NOT NULL,                       -- commitment to the note details (recipient + assets); primary key
    note_id TEXT NOT NULL,                                  -- the full note id (hash(details_commitment, metadata_commitment))
    recipient_digest TEXT NOT NULL,                                -- the note recipient
    assets BLOB NOT NULL,                                   -- the serialized NoteAssets, including vault commitment and list of assets
    metadata BLOB NOT NULL,                                 -- serialized metadata
    nullifier TEXT NULL,
    expected_height UNSIGNED INT NOT NULL,                  -- the block height after which the note is expected to be created
-- TODO: normalize script data for output notes
--     script_commitment TEXT NULL,
    state_discriminant UNSIGNED INT NOT NULL,               -- state discriminant of the note, used to query by state
    state BLOB NOT NULL,                                    -- serialized note state
    attachments BLOB NOT NULL,

    PRIMARY KEY (details_commitment)
) WITHOUT ROWID;
CREATE INDEX idx_output_notes_state ON output_notes(state_discriminant);
CREATE INDEX idx_output_notes_nullifier ON output_notes(nullifier);
CREATE INDEX idx_output_notes_note_id ON output_notes(note_id);

CREATE TABLE notes_scripts (
    script_root TEXT NOT NULL,                       -- Note script root
    serialized_note_script BLOB,                     -- NoteScript, serialized

    PRIMARY KEY (script_root)
);

-- ── Blockchain checkpoint & tags ─────────────────────────────────────────

CREATE TABLE blockchain_checkpoint (
    block_num UNSIGNED BIG INT NOT NULL,    -- the block number of the most recent state sync
    partial_blockchain_peaks BLOB NOT NULL, -- serialized MMR peaks at the current sync height
    PRIMARY KEY (block_num)
);

CREATE TABLE tags (
    tag BLOB NOT NULL,     -- the serialized tag
    source BLOB NOT NULL   -- the serialized tag source
);

-- insert initial row into blockchain_checkpoint table
INSERT OR IGNORE INTO blockchain_checkpoint (block_num, partial_blockchain_peaks)
SELECT 0, X''
WHERE (
    SELECT COUNT(*) FROM blockchain_checkpoint
) = 0;

-- ── Block headers & partial blockchain ───────────────────────────────────

CREATE TABLE block_headers (
    block_num UNSIGNED BIG INT NOT NULL,  -- block number
    header BLOB NOT NULL,                 -- serialized block header
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

-- ── PSWAP lineage tracking ───────────────────────────────────────────────
--
-- One row per PSWAP order created by an account this client tracks. The
-- order is identified by `order_id = original_pswap.serial[1]`, which is
-- stable across every fill round in the chain.
--
-- The `original_pswap` BLOB carries the serialised PswapNote that the
-- creator emitted. Every "initial" field (creator/sender, offered &
-- requested assets, serial number, note types) is derived on demand by
-- deserialising this BLOB and calling the existing PswapNote getters.
-- `order_id` is duplicated outside the BLOB as a primary key so the
-- per-note observer can look it up without deserialising on every
-- incoming chain note.
--
-- The mutable columns describe the live tip of the chain at any point:
-- which note is at the head, how much of the offered / requested totals
-- is still unfilled, and what depth we have advanced through. The
-- `last_consumer_account_id` / `last_payout_amount` columns are needed
-- to reconstruct the current tip via `PswapNote::remainder_note(...)`
-- when the creator wants to reclaim a chain that has been partially
-- filled by other accounts (the creator never originated the remainder,
-- so we need enough information to rebuild it byte-identically).
-- Storage-format note: this table mixes BLOB (for Felt-/AccountId-shaped
-- keys: `order_id`, `last_consumer_account_id`, `original_pswap`) and TEXT
-- (for hash-shaped keys: `current_tip_note_id`, `current_tip_nullifier`).
-- PSWAP chain tracking: one row per tracked order. The current tip note
-- itself lives in `input_notes` (for remainders; depth > 0) or
-- `output_notes` (for the original; depth == 0). `current_tip_note_id`
-- is the lookup key into those tables — reclaim flow reads the tip from
-- there rather than reconstructing it.
CREATE TABLE pswap_lineages (
    order_id                  BLOB    NOT NULL,  -- Felt (8 bytes), == original_pswap.serial[1]
    original_pswap            BLOB    NOT NULL,  -- serialised PswapNote (source of truth for every initial-* field)

    -- Live tip state.
    current_tip_note_id       TEXT    NOT NULL,                  -- hex; matches sibling tables' note_id format; indexed for fast lookup during sync
    current_depth             UNSIGNED BIG INT NOT NULL,         -- u32, 0 for the original tip
    remaining_offered         UNSIGNED BIG INT NOT NULL,         -- AssetAmount as u64; validated <= AssetAmount::MAX on read
    remaining_requested       UNSIGNED BIG INT NOT NULL,         -- AssetAmount as u64; validated <= AssetAmount::MAX on read

    state                     UNSIGNED INT NOT NULL,             -- PswapLineageState discriminant: Active=0, FullyFilled=1, Reclaimed=2
    created_at_block          UNSIGNED BIG INT NOT NULL,
    updated_at_block          UNSIGNED BIG INT NOT NULL,

    PRIMARY KEY (order_id)
) WITHOUT ROWID;

CREATE INDEX idx_pswap_lineages_state         ON pswap_lineages(state);
CREATE INDEX idx_pswap_lineages_tip_note_id   ON pswap_lineages(current_tip_note_id);
