-- Add up migration script here
CREATE TABLE IF NOT EXISTS battle_statuses (
	id varchar(255) NOT NULL PRIMARY KEY,
	user_id varchar(255) NOT NULL REFERENCES users(id),
	display_name varchar(255) NOT NULL,
	status varchar(255) NOT NULL,
	connected boolean NOT NULL DEFAULT TRUE,
	created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);