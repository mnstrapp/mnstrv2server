-- Add up migration script here
CREATE TABLE IF NOT EXISTS transactions (
	id varchar(255) NOT NULL,
	wallet_id varchar(255) NOT NULL,
	transaction_type varchar(255) NOT NULL,
	transaction_amount int8 NOT NULL,
	transaction_status varchar(255) NOT NULL,
	transaction_data text NULL,
	error_message text NULL,
	created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	CONSTRAINT transactions_pkey PRIMARY KEY (id),
	CONSTRAINT transactions_wallet_id_fkey FOREIGN KEY (wallet_id) REFERENCES wallets(id)
);
CREATE INDEX IF NOT EXISTS idx_transactions_created_at ON transactions USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_transactions_error_message ON transactions USING btree (error_message);
CREATE INDEX IF NOT EXISTS idx_transactions_id ON transactions USING btree (id);
CREATE INDEX IF NOT EXISTS idx_transactions_transaction_amount ON transactions USING btree (transaction_amount);
CREATE INDEX IF NOT EXISTS idx_transactions_transaction_status ON transactions USING btree (transaction_status);
CREATE INDEX IF NOT EXISTS idx_transactions_transaction_type ON transactions USING btree (transaction_type);
CREATE INDEX IF NOT EXISTS idx_transactions_updated_at ON transactions USING btree (updated_at);
CREATE INDEX IF NOT EXISTS idx_transactions_wallet_id ON transactions USING btree (wallet_id);