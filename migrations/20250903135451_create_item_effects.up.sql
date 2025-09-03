-- Add up migration script here
CREATE TABLE IF NOT EXISTS item_effects (
	id varchar(255) NOT NULL,
	item_id varchar(255) NOT NULL,
	effect_id varchar(255) NOT NULL,
	created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	archived_at timestamp with time zone NULL,
	CONSTRAINT item_effects_pkey PRIMARY KEY (id),
	CONSTRAINT item_effects_effect_id_fkey FOREIGN KEY (effect_id) REFERENCES effects(id),
	CONSTRAINT item_effects_item_id_fkey FOREIGN KEY (item_id) REFERENCES items(id)
);
CREATE INDEX IF NOT EXISTS idx_item_effects_archived_at ON item_effects USING btree (archived_at);
CREATE INDEX IF NOT EXISTS idx_item_effects_created_at ON item_effects USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_item_effects_effect_id ON item_effects USING btree (effect_id);
CREATE INDEX IF NOT EXISTS idx_item_effects_id ON item_effects USING btree (id);
CREATE INDEX IF NOT EXISTS idx_item_effects_item_id ON item_effects USING btree (item_id);
CREATE INDEX IF NOT EXISTS idx_item_effects_updated_at ON item_effects USING btree (updated_at);