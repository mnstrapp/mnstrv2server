-- Add up migration script here
CREATE TABLE IF NOT EXISTS mnstr_user_items (
	id varchar(255) NOT NULL,
	user_item_id varchar(255) NOT NULL,
	mnstr_id varchar(255) NOT NULL,
	created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	archived_at timestamp with time zone NULL,
	CONSTRAINT mnstr_user_items_pkey PRIMARY KEY (id),
	CONSTRAINT mnstr_user_items_mnstr_id_fkey FOREIGN KEY (mnstr_id) REFERENCES mnstrs(id),
	CONSTRAINT mnstr_user_items_user_item_id_fkey FOREIGN KEY (user_item_id) REFERENCES user_items(id)
);
CREATE INDEX IF NOT EXISTS idx_mnstr_user_items_archived_at ON mnstr_user_items USING btree (archived_at);
CREATE INDEX IF NOT EXISTS idx_mnstr_user_items_created_at ON mnstr_user_items USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_mnstr_user_items_id ON mnstr_user_items USING btree (id);
CREATE INDEX IF NOT EXISTS idx_mnstr_user_items_mnstr_id ON mnstr_user_items USING btree (mnstr_id);
CREATE INDEX IF NOT EXISTS idx_mnstr_user_items_updated_at ON mnstr_user_items USING btree (updated_at);
CREATE INDEX IF NOT EXISTS idx_mnstr_user_items_user_item_id ON public.mnstr_user_items USING btree (user_item_id);