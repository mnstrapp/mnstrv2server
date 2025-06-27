-- +goose Up
-- +goose StatementBegin
CREATE TABLE
    IF NOT EXISTS mnstrs (
        id VARCHAR(255) PRIMARY KEY,
        user_id VARCHAR(255) NOT NULL REFERENCES users (id),
        mnstr_name VARCHAR(255),
        mnstr_description TEXT,
        mnstr_qr_code TEXT NOT NULL,
        created_at TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW (),
            updated_at TIMESTAMP
        WITH
            TIME ZONE NOT NULL DEFAULT NOW (),
            archived_at TIMESTAMP
        WITH
            TIME ZONE
    );

CREATE INDEX IF NOT EXISTS idx_mnstrs_id ON mnstrs (id);

CREATE INDEX IF NOT EXISTS idx_mnstrs_user_id ON mnstrs (user_id);

CREATE INDEX IF NOT EXISTS idx_mnstrs_mnstr_qr_code ON mnstrs (mnstr_qr_code);

CREATE INDEX IF NOT EXISTS idx_mnstrs_mnstr_name ON mnstrs (mnstr_name);

CREATE INDEX IF NOT EXISTS idx_mnstrs_created_at ON mnstrs (created_at);

CREATE INDEX IF NOT EXISTS idx_mnstrs_updated_at ON mnstrs (updated_at);

CREATE INDEX IF NOT EXISTS idx_mnstrs_archived_at ON mnstrs (archived_at);

-- +goose StatementEnd
-- +goose Down
-- +goose StatementBegin
DROP INDEX IF EXISTS idx_mnstrs_id;

DROP INDEX IF EXISTS idx_mnstrs_user_id;

DROP INDEX IF EXISTS idx_mnstrs_mnstr_qr_code;

DROP INDEX IF EXISTS idx_mnstrs_mnstr_name;

DROP INDEX IF EXISTS idx_mnstrs_created_at;

DROP INDEX IF EXISTS idx_mnstrs_updated_at;

DROP INDEX IF EXISTS idx_mnstrs_archived_at;

DROP TABLE IF EXISTS mnstrs;

-- +goose StatementEnd