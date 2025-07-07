-- +goose Up
-- +goose StatementBegin
ALTER TABLE users ADD COLUMN experience_level INTEGER DEFAULT 0;
ALTER TABLE users ADD COLUMN experience_points INTEGER DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_users_experience_level ON users (experience_level);
CREATE INDEX IF NOT EXISTS idx_users_experience_points ON users (experience_points);
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
DROP INDEX IF EXISTS idx_users_experience_level;
DROP INDEX IF EXISTS idx_users_experience_points;

ALTER TABLE users DROP COLUMN experience_level;
ALTER TABLE users DROP COLUMN experience_points;
-- +goose StatementEnd
