-- Add down migration script here
ALTER TABLE battle_statuses
ADD COLUMN connected boolean NOT NULL DEFAULT TRUE;