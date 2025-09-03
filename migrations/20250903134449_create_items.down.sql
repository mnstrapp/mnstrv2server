-- Add down migration script here
DROP INDEX IF EXISTS idx_items_archived_at;
DROP INDEX IF EXISTS idx_items_created_at;
DROP INDEX IF EXISTS idx_items_id;
DROP INDEX IF EXISTS idx_items_item_description;
DROP INDEX IF EXISTS idx_items_item_image;
DROP INDEX IF EXISTS idx_items_item_name;
DROP INDEX IF EXISTS idx_items_item_price;
DROP INDEX IF EXISTS idx_items_updated_at;
DROP TABLE IF EXISTS items;