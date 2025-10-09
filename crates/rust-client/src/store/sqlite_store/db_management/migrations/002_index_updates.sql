
CREATE INDEX IF NOT EXISTS idx_accounts_id ON accounts(id);

CREATE INDEX IF NOT EXISTS idx_input_notes_nullifier ON input_notes(nullifier);

CREATE INDEX IF NOT EXISTS idx_output_notes_nullifier ON output_notes(nullifier);

CREATE INDEX IF NOT EXISTS idx_block_headers_has_notes ON block_headers(block_num) WHERE has_client_notes = 1;

CREATE INDEX IF NOT EXISTS idx_transactions_uncommitted ON transactions(status_variant);

-- Create tracked_accounts table to easily read account IDs
-- TODO: this should maybe use the settings table in the future?

CREATE TABLE IF NOT EXISTS tracked_accounts (
    id TEXT NOT NULL PRIMARY KEY
);

INSERT OR IGNORE INTO tracked_accounts (id)
SELECT DISTINCT id FROM accounts;
