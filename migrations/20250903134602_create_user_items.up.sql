-- Add up migration script here
CREATE TABLE IF NOT EXISTS user_items (
	id varchar(255) NOT NULL,
	user_id varchar(255) NOT NULL,
	item_id varchar(255) NOT NULL,
	created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	archived_at timestamp with time zone NULL,
	CONSTRAINT user_items_pkey PRIMARY KEY (id),
	CONSTRAINT user_items_item_id_fkey FOREIGN KEY (item_id) REFERENCES items(id),
	CONSTRAINT user_items_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id)
);
CREATE INDEX IF NOT EXISTS idx_user_items_archived_at ON user_items USING btree (archived_at);
CREATE INDEX IF NOT EXISTS idx_user_items_created_at ON user_items USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_user_items_id ON user_items USING btree (id);
CREATE INDEX IF NOT EXISTS idx_user_items_item_id ON user_items USING btree (item_id);
CREATE INDEX IF NOT EXISTS idx_user_items_updated_at ON user_items USING btree (updated_at);
CREATE INDEX IF NOT EXISTS idx_user_items_user_id ON user_items USING btree (user_id);