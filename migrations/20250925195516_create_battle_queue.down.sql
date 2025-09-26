-- Add down migration script here
DROP TRIGGER IF EXISTS battle_queue_notify_trigger ON battle_queue;
DROP FUNCTION IF EXISTS battle_queue_notify();
DROP TABLE IF EXISTS battle_queue;