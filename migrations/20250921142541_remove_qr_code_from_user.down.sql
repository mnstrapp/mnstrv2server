-- Add down migration script here
ALTER TABLE users ADD COLUMN qr_code text NOT NULL;