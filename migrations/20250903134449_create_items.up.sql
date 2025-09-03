-- Add up migration script here
CREATE TABLE IF NOT EXISTS items (
	id varchar(255) NOT NULL,
	item_name varchar(255) NOT NULL,
	item_description text NULL,
	item_price int4 NOT NULL,
	item_image text NULL,
	created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	archived_at timestamp with time zone NULL,
	CONSTRAINT items_item_name_key UNIQUE (item_name),
	CONSTRAINT items_pkey PRIMARY KEY (id)
);
CREATE INDEX IF NOT EXISTS idx_items_archived_at ON items USING btree (archived_at);
CREATE INDEX IF NOT EXISTS idx_items_created_at ON items USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_items_id ON items USING btree (id);
CREATE INDEX IF NOT EXISTS idx_items_item_description ON items USING btree (item_description);
CREATE INDEX IF NOT EXISTS idx_items_item_image ON items USING btree (item_image);
CREATE INDEX IF NOT EXISTS idx_items_item_name ON items USING btree (item_name);
CREATE INDEX IF NOT EXISTS idx_items_item_price ON items USING btree (item_price);
CREATE INDEX IF NOT EXISTS idx_items_updated_at ON items USING btree (updated_at);