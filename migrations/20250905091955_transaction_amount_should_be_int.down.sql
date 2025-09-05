-- Add down migration script here
ALTER TABLE transactions ALTER COLUMN transaction_amount TYPE int8;