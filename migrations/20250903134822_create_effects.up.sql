-- Add up migration script here
CREATE TABLE IF NOT EXISTS effects (
	id varchar(255) NOT NULL,
	effect_skill varchar(255) NOT NULL,
	effect_value int4 NOT NULL,
	effect_duration int4 DEFAULT 0 NULL,
	effect_name varchar(255) NOT NULL,
	effect_description text NULL,
	effect_image text NULL,
	created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	archived_at timestamp with time zone NULL,
	CONSTRAINT effects_effect_name_key UNIQUE (effect_name),
	CONSTRAINT effects_pkey PRIMARY KEY (id)
);
CREATE INDEX IF NOT EXISTS idx_effects_archived_at ON effects USING btree (archived_at);
CREATE INDEX IF NOT EXISTS idx_effects_created_at ON effects USING btree (created_at);
CREATE INDEX IF NOT EXISTS idx_effects_effect_description ON effects USING btree (effect_description);
CREATE INDEX IF NOT EXISTS idx_effects_effect_duration ON effects USING btree (effect_duration);
CREATE INDEX IF NOT EXISTS idx_effects_effect_image ON effects USING btree (effect_image);
CREATE INDEX IF NOT EXISTS idx_effects_effect_name ON effects USING btree (effect_name);
CREATE INDEX IF NOT EXISTS idx_effects_effect_skill ON effects USING btree (effect_skill);
CREATE INDEX IF NOT EXISTS idx_effects_effect_value ON effects USING btree (effect_value);
CREATE INDEX IF NOT EXISTS idx_effects_id ON effects USING btree (id);
CREATE INDEX IF NOT EXISTS idx_effects_updated_at ON effects USING btree (updated_at);