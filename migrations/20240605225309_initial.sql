-- Add migration script here
-- create user table
CREATE TABLE IF NOT EXISTS users(
  id bigserial PRIMARY KEY,
  -- ws_id bigint NOT NULL,
  name varchar(64) NOT NULL,
  email varchar(64) NOT NULL,
  -- hashed argon2 password, length 97
  password_hash varchar(97) NOT NULL,
  created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);



-- create index for users for email
CREATE UNIQUE INDEX IF NOT EXISTS email_index ON users(email);
