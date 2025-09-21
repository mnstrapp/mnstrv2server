-- Add up migration script here
ALTER TABLE users ADD COLUMN phone VARCHAR(255) NULL;
ALTER TABLE users ADD CONSTRAINT users_phone_key UNIQUE (phone);
CREATE INDEX IF NOT EXISTS idx_users_phone ON users (phone);
ALTER TABLE users ADD COLUMN phone_verified BOOLEAN DEFAULT FALSE NULL;
ALTER TABLE users ADD COLUMN email_verified BOOLEAN DEFAULT FALSE NULL;
ALTER TABLE users DROP CONSTRAINT users_email_key;
ALTER TABLE users ADD CONSTRAINT users_email_key UNIQUE (email);
ALTER TABLE users ALTER COLUMN email DROP NOT NULL;