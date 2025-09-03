-- Add down migration script here
DROP INDEX IF EXISTS idx_item_effects_archived_at;
DROP INDEX IF EXISTS idx_item_effects_created_at;
DROP INDEX IF EXISTS idx_item_effects_effect_id;
DROP INDEX IF EXISTS idx_item_effects_id;
DROP INDEX IF EXISTS idx_item_effects_item_id;
DROP INDEX IF EXISTS idx_item_effects_updated_at;
DROP TABLE IF EXISTS item_effects;