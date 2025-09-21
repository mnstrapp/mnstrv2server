-- Add down migration script here
ALTER TABLE users DROP COLUMN phone;
ALTER TABLE users DROP CONSTRAINT users_phone_key;
DROP INDEX IF EXISTS idx_users_phone;
ALTER TABLE users DROP COLUMN email_verified;
ALTER TABLE users DROP COLUMN phone_verified;
ALTER TABLE users ADD CONSTRAINT users_email_key UNIQUE (email);
ALTER TABLE users ALTER COLUMN email SET NOT NULL;