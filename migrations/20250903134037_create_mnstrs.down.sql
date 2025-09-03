-- Add down migration script here
DROP INDEX IF EXISTS idx_mnstrs_archived_at;
DROP INDEX IF EXISTS idx_mnstrs_created_at;
DROP INDEX IF EXISTS idx_mnstrs_id;
DROP INDEX IF EXISTS idx_mnstrs_mnstr_name;
DROP INDEX IF EXISTS idx_mnstrs_mnstr_qr_code;
DROP INDEX IF EXISTS idx_mnstrs_updated_at;
DROP INDEX IF EXISTS idx_mnstrs_user_id;
DROP TABLE IF EXISTS mnstrs;