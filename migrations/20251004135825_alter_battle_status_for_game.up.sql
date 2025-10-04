-- Add up migration script here
ALTER TABLE battle_statuses
ADD COLUMN opponent_id varchar(255) NULL REFERENCES users(id);
ALTER TABLE battle_statuses
ADD COLUMN opponent_name varchar(255) NULL;
ALTER TABLE battle_statuses
ADD COLUMN battle_id varchar(255) NULL REFERENCES battles(id);