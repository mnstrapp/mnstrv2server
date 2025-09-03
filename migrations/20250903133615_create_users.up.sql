-- Add up migration script here
CREATE TABLE IF NOT EXISTS users (
	id varchar(255) NOT NULL,
	display_name varchar(255) NOT NULL,
	email varchar(255) NOT NULL,
	password_hash varchar(255) NOT NULL,
	qr_code text NOT NULL,
	created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	archived_at timestamp with time zone NULL,
	experience_level int4 DEFAULT 0 NULL,
	experience_points int4 DEFAULT 0 NULL,
	CONSTRAINT users_display_name_key UNIQUE (display_name),
	CONSTRAINT users_email_key UNIQUE (email),
	CONSTRAINT users_pkey PRIMARY KEY (id),
	CONSTRAINT users_qr_code_key UNIQUE (qr_code)
);
CREATE INDEX IF NOT EXISTS idx_users_archived_at ON users USING btree (archived_at);
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_users_display_name ON users USING btree (display_name);
CREATE INDEX IF NOT EXISTS idx_users_email ON users USING btree (email);
CREATE INDEX IF NOT EXISTS idx_users_experience_level ON users USING btree (experience_level);
CREATE INDEX IF NOT EXISTS idx_users_experience_points ON users USING btree (experience_points);
CREATE INDEX IF NOT EXISTS idx_users_qr_code ON users USING btree (qr_code);
CREATE INDEX IF NOT EXISTS idx_users_updated_at ON users USING btree (updated_at);