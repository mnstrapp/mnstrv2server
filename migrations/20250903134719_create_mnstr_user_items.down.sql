-- Add down migration script here
DROP INDEX IF EXISTS idx_mnstr_user_items_archived_at;
DROP INDEX IF EXISTS idx_mnstr_user_items_created_at;
DROP INDEX IF EXISTS idx_mnstr_user_items_id;
DROP INDEX IF EXISTS idx_mnstr_user_items_mnstr_id;
DROP INDEX IF EXISTS idx_mnstr_user_items_updated_at;
DROP INDEX IF EXISTS idx_mnstr_user_items_user_item_id;
DROP TABLE IF EXISTS mnstr_user_items;