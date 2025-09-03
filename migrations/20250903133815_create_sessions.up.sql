-- Add up migration script here
CREATE TABLE IF NOT EXISTS sessions (
	id varchar(255) NOT NULL,
	user_id varchar(255) NOT NULL,
	session_token varchar(255) NOT NULL,
	created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	archived_at timestamp with time zone NULL,
	expires_at timestamp with time zone NOT NULL,
	CONSTRAINT sessions_pkey PRIMARY KEY (id),
	CONSTRAINT sessions_session_token_key UNIQUE (session_token),
	CONSTRAINT sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id)
);
CREATE INDEX IF NOT EXISTS idx_sessions_archived_at ON sessions USING btree (archived_at);
CREATE INDEX IF NOT EXISTS idx_sessions_created_at ON sessions USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions USING btree (expires_at);
CREATE INDEX IF NOT EXISTS idx_sessions_session_token ON sessions USING btree (session_token);
CREATE INDEX IF NOT EXISTS idx_sessions_updated_at ON sessions USING btree (updated_at);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions USING btree (user_id);