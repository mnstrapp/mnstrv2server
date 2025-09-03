-- Add down migration script here
DROP INDEX IF EXISTS idx_transactions_created_at;
DROP INDEX IF EXISTS idx_transactions_error_message;
DROP INDEX IF EXISTS idx_transactions_id;
DROP INDEX IF EXISTS idx_transactions_transaction_amount;
DROP INDEX IF EXISTS idx_transactions_transaction_status;
DROP INDEX IF EXISTS idx_transactions_transaction_type;
DROP INDEX IF EXISTS idx_transactions_updated_at;
DROP INDEX IF EXISTS idx_transactions_wallet_id;
DROP TABLE IF EXISTS transactions;