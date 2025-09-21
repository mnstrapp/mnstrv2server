-- Add up migration script here
ALTER TABLE users ADD COLUMN email_verification_code VARCHAR(255) NULL;
ALTER TABLE users ADD COLUMN phone_verification_code VARCHAR(255) NULL;