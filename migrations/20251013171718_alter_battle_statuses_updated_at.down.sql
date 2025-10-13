-- Add down migration script here
ALTER TABLE battle_statuses
DROP COLUMN updated_at;