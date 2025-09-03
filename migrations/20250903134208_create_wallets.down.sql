-- Add down migration script here
DROP INDEX IF EXISTS idx_wallets_archived_at;
DROP INDEX IF EXISTS idx_wallets_created_at;
DROP INDEX IF EXISTS idx_wallets_id;
DROP INDEX IF EXISTS idx_wallets_updated_at;
DROP INDEX IF EXISTS idx_wallets_user_id;
DROP TABLE IF EXISTS wallets;