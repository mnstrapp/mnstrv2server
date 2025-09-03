-- Add up migration script here
CREATE TABLE IF NOT EXISTS mnstrs (
	id varchar(255) NOT NULL,
	user_id varchar(255) NOT NULL,
	mnstr_name varchar(255) NULL,
	mnstr_description text NULL,
	mnstr_qr_code text NOT NULL,
	created_at timestamp with time zone DEFAULT now() NOT NULL,
	updated_at timestamp with time zone DEFAULT now() NOT NULL,
	archived_at timestamp with time zone NULL,
	current_level int4 DEFAULT 0 NOT NULL,
	current_experience int4 DEFAULT 0 NOT NULL,
	current_health int4 DEFAULT 100 NULL,
	max_health int4 DEFAULT 100 NULL,
	current_attack int4 DEFAULT 10 NULL,
	max_attack int4 DEFAULT 10 NULL,
	current_defense int4 DEFAULT 10 NULL,
	max_defense int4 DEFAULT 10 NULL,
	current_speed int4 DEFAULT 10 NULL,
	max_speed int4 DEFAULT 10 NULL,
	current_intelligence int4 DEFAULT 10 NULL,
	max_intelligence int4 DEFAULT 10 NULL,
	current_magic int4 DEFAULT 10 NULL,
	max_magic int4 DEFAULT 10 NULL,
	CONSTRAINT mnstrs_pkey PRIMARY KEY (id),
	CONSTRAINT mnstrs_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id)
);
CREATE INDEX IF NOT EXISTS idx_mnstrs_archived_at ON mnstrs USING btree (archived_at);
CREATE INDEX IF NOT EXISTS idx_mnstrs_created_at ON mnstrs USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_mnstrs_id ON mnstrs USING btree (id);
CREATE INDEX IF NOT EXISTS idx_mnstrs_mnstr_name ON mnstrs USING btree (mnstr_name);
CREATE INDEX IF NOT EXISTS idx_mnstrs_mnstr_qr_code ON mnstrs USING btree (mnstr_qr_code);
CREATE INDEX IF NOT EXISTS idx_mnstrs_updated_at ON mnstrs USING btree (updated_at);
CREATE INDEX IF NOT EXISTS idx_mnstrs_user_id ON mnstrs USING btree (user_id);