-- Add up migration script here
CREATE TABLE IF NOT EXISTS battles (
	id varchar(255) NOT NULL PRIMARY KEY,
	challenger_id varchar(255) NOT NULL REFERENCES users(id),
	challenger_name varchar(255) NOT NULL,
	challenger_mnstr_id varchar(255) NULL REFERENCES mnstrs(id),
	opponent_id varchar(255) NOT NULL REFERENCES users(id),
	opponent_name varchar(255) NOT NULL,
	opponent_mnstr_id varchar(255) NULL REFERENCES mnstrs(id),
	winner_id varchar(255) NULL REFERENCES users(id),
	winner_mnstr_id varchar(255) NULL REFERENCES mnstrs(id),
	created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	archived_at timestamp with time zone NULL
);

CREATE TABLE IF NOT EXISTS battle_logs (
	id varchar(255) NOT NULL PRIMARY KEY,
	battle_id varchar(255) NOT NULL REFERENCES battles(id),
	user_id varchar(255) NOT NULL REFERENCES users(id),
	mnstr_id varchar(255) NOT NULL REFERENCES mnstrs(id),
    action varchar(255) NOT NULL,
	data text NOT NULL,
	created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);