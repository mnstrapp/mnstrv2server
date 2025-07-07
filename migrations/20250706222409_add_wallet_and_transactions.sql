-- +goose Up
-- +goose StatementBegin
CREATE TABLE IF NOT EXISTS wallets (
    id VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL REFERENCES users(id) UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    archived_at TIMESTAMP WITH TIME ZONE DEFAULT NULL
);

CREATE INDEX IF NOT EXISTS idx_wallets_id ON wallets(id);
CREATE INDEX IF NOT EXISTS idx_wallets_user_id ON wallets(user_id);
CREATE INDEX IF NOT EXISTS idx_wallets_created_at ON wallets(created_at);
CREATE INDEX IF NOT EXISTS idx_wallets_updated_at ON wallets(updated_at);
CREATE INDEX IF NOT EXISTS idx_wallets_archived_at ON wallets(archived_at);

CREATE TABLE IF NOT EXISTS transactions (
    id VARCHAR(255) PRIMARY KEY,
    wallet_id VARCHAR(255) NOT NULL REFERENCES wallets(id),
    user_id VARCHAR(255) NOT NULL,
    transaction_type VARCHAR(255) NOT NULL,
    transaction_amount BIGINT NOT NULL,
    transaction_status VARCHAR(255) NOT NULL,
    transaction_data TEXT,
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_transactions_id ON transactions(id);
CREATE INDEX IF NOT EXISTS idx_transactions_wallet_id ON transactions(wallet_id);
CREATE INDEX IF NOT EXISTS idx_transactions_user_id ON transactions(user_id);
CREATE INDEX IF NOT EXISTS idx_transactions_transaction_type ON transactions(transaction_type);
CREATE INDEX IF NOT EXISTS idx_transactions_transaction_amount ON transactions(transaction_amount);
CREATE INDEX IF NOT EXISTS idx_transactions_transaction_status ON transactions(transaction_status);
CREATE INDEX IF NOT EXISTS idx_transactions_error_message ON transactions(error_message);
CREATE INDEX IF NOT EXISTS idx_transactions_created_at ON transactions(created_at);
CREATE INDEX IF NOT EXISTS idx_transactions_updated_at ON transactions(updated_at);

-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin


DROP INDEX IF EXISTS idx_transactions_id;
DROP INDEX IF EXISTS idx_transactions_wallet_id;
DROP INDEX IF EXISTS idx_transactions_user_id;
DROP INDEX IF EXISTS idx_transactions_transaction_type;
DROP INDEX IF EXISTS idx_transactions_transaction_amount;
DROP INDEX IF EXISTS idx_transactions_transaction_status;
DROP INDEX IF EXISTS idx_transactions_error_message;
DROP INDEX IF EXISTS idx_transactions_created_at;
DROP INDEX IF EXISTS idx_transactions_updated_at;

DROP TABLE IF EXISTS transactions;

DROP INDEX IF EXISTS idx_wallets_id;
DROP INDEX IF EXISTS idx_wallets_user_id;
DROP INDEX IF EXISTS idx_wallets_created_at;
DROP INDEX IF EXISTS idx_wallets_updated_at;
DROP INDEX IF EXISTS idx_wallets_archived_at;

DROP TABLE IF EXISTS wallets;
-- +goose StatementEnd
