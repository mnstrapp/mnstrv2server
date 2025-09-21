-- Add down migration script here
ALTER TABLE users DROP COLUMN email_verification_code;
ALTER TABLE users DROP COLUMN phone_verification_code;