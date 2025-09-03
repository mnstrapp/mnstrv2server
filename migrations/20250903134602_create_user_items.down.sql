-- Add down migration script here
DROP INDEX IF EXISTS idx_user_items_archived_at;
DROP INDEX IF EXISTS idx_user_items_created_at;
DROP INDEX IF EXISTS idx_user_items_id;
DROP INDEX IF EXISTS idx_user_items_item_id;
DROP INDEX IF EXISTS idx_user_items_updated_at;
DROP INDEX IF EXISTS idx_user_items_user_id;
DROP TABLE IF EXISTS user_items;