-- Add down migration script here
ALTER TABLE battle_statuses
DROP COLUMN opponent_id;
ALTER TABLE battle_statuses
DROP COLUMN opponent_name;
ALTER TABLE battle_statuses
DROP COLUMN battle_id;