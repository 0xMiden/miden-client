-- Watched accounts: public accounts the client monitors but does not own.
CREATE TABLE watched_accounts (
    id                TEXT NOT NULL PRIMARY KEY,
    account_header    BLOB NOT NULL,
    code_commitment   TEXT NOT NULL,
    storage_header    BLOB NOT NULL,
    last_synced_block UNSIGNED BIG INT NOT NULL,
    FOREIGN KEY (code_commitment) REFERENCES account_code(commitment)
);
