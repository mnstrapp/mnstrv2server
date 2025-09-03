-- Add down migration script here
DROP INDEX IF EXISTS idx_effects_archived_at;
DROP INDEX IF EXISTS idx_effects_created_at;
DROP INDEX IF EXISTS idx_effects_effect_description;
DROP INDEX IF EXISTS idx_effects_effect_duration;
DROP INDEX IF EXISTS idx_effects_effect_image;
DROP INDEX IF EXISTS idx_effects_effect_name;
DROP INDEX IF EXISTS idx_effects_effect_skill;
DROP INDEX IF EXISTS idx_effects_effect_value;
DROP INDEX IF EXISTS idx_effects_id;
DROP INDEX IF EXISTS idx_effects_updated_at;
DROP TABLE IF EXISTS effects;