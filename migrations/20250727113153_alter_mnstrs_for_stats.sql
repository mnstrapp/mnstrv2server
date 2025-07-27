-- +goose Up
-- +goose StatementBegin
ALTER TABLE mnstrs ADD COLUMN current_level INTEGER NOT NULL DEFAULT 0;
ALTER TABLE mnstrs ADD COLUMN current_experience INTEGER NOT NULL DEFAULT 0;
ALTER TABLE mnstrs ADD COLUMN current_health INTEGER NULL DEFAULT 100;
ALTER TABLE mnstrs ADD COLUMN max_health INTEGER NULL DEFAULT 100;
ALTER TABLE mnstrs ADD COLUMN current_attack INTEGER NULL DEFAULT 10;
ALTER TABLE mnstrs ADD COLUMN max_attack INTEGER NULL DEFAULT 10;
ALTER TABLE mnstrs ADD COLUMN current_defense INTEGER NULL DEFAULT 10;
ALTER TABLE mnstrs ADD COLUMN max_defense INTEGER NULL DEFAULT 10;
ALTER TABLE mnstrs ADD COLUMN current_speed INTEGER NULL DEFAULT 10;
ALTER TABLE mnstrs ADD COLUMN max_speed INTEGER NULL DEFAULT 10;
ALTER TABLE mnstrs ADD COLUMN current_intelligence INTEGER NULL DEFAULT 10;
ALTER TABLE mnstrs ADD COLUMN max_intelligence INTEGER NULL DEFAULT 10;
ALTER TABLE mnstrs ADD COLUMN current_magic INTEGER NULL DEFAULT 10;
ALTER TABLE mnstrs ADD COLUMN max_magic INTEGER NULL DEFAULT 10;
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
ALTER TABLE mnstrs DROP COLUMN current_level;
ALTER TABLE mnstrs DROP COLUMN current_experience;
ALTER TABLE mnstrs DROP COLUMN current_health;
ALTER TABLE mnstrs DROP COLUMN max_health;
ALTER TABLE mnstrs DROP COLUMN current_attack;
ALTER TABLE mnstrs DROP COLUMN max_attack;
ALTER TABLE mnstrs DROP COLUMN current_defense;
ALTER TABLE mnstrs DROP COLUMN max_defense;
ALTER TABLE mnstrs DROP COLUMN current_speed;
ALTER TABLE mnstrs DROP COLUMN max_speed;
ALTER TABLE mnstrs DROP COLUMN current_intelligence;
ALTER TABLE mnstrs DROP COLUMN max_intelligence;
ALTER TABLE mnstrs DROP COLUMN current_magic;
ALTER TABLE mnstrs DROP COLUMN max_magic;
-- +goose StatementEnd
