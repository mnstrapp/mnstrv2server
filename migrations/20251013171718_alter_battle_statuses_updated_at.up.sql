-- Add up migration script here
ALTER TABLE battle_statuses
ADD COLUMN updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL;