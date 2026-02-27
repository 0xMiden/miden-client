-- Rework watched accounts: store them in the existing account tables with a `watched` flag
-- instead of a separate table.
DROP TABLE IF EXISTS watched_accounts;
ALTER TABLE latest_account_headers ADD COLUMN watched BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE historical_account_headers ADD COLUMN watched BOOLEAN NOT NULL DEFAULT FALSE;
