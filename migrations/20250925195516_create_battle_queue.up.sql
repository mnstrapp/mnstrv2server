-- Add up migration script here
CREATE TABLE IF NOT EXISTS battle_queue (
	id varchar(255) PRIMARY KEY,
	user_id varchar(255) NOT NULL REFERENCES users(id),
	channel text NOT NULL,
    action varchar(255) NOT NULL,
    data jsonb NOT NULL,
	created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	archived_at timestamp with time zone NULL
);

CREATE OR REPLACE FUNCTION battle_queue_notify() RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify(cast(NEW.channel as text), row_to_json(NEW)::text);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER battle_queue_notify_trigger AFTER INSERT ON battle_queue FOR EACH ROW EXECUTE FUNCTION battle_queue_notify();