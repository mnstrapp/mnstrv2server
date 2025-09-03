-- Add down migration script here
DROP INDEX IF EXISTS idx_users_archived_at;
DROP INDEX IF EXISTS idx_users_created_at;
DROP INDEX IF EXISTS idx_users_display_name;
DROP INDEX IF EXISTS idx_users_email;
DROP INDEX IF EXISTS idx_users_experience_level;
DROP INDEX IF EXISTS idx_users_experience_points;
DROP INDEX IF EXISTS idx_users_qr_code;
DROP INDEX IF EXISTS idx_users_updated_at;
DROP TABLE IF EXISTS public.users;