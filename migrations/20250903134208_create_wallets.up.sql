-- Add up migration script here
CREATE TABLE IF NOT EXISTS wallets (
	id varchar(255) NOT NULL,
	user_id varchar(255) NOT NULL,
	created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	archived_at timestamp with time zone NULL,
	CONSTRAINT wallets_pkey PRIMARY KEY (id),
	CONSTRAINT wallets_user_id_key UNIQUE (user_id),
	CONSTRAINT wallets_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id)
);
CREATE INDEX IF NOT EXISTS idx_wallets_archived_at ON wallets USING btree (archived_at);
CREATE INDEX IF NOT EXISTS idx_wallets_created_at ON wallets USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_wallets_id ON wallets USING btree (id);
CREATE INDEX IF NOT EXISTS idx_wallets_updated_at ON wallets USING btree (updated_at);
CREATE INDEX IF NOT EXISTS idx_wallets_user_id ON wallets USING btree (user_id);