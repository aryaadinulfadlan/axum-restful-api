-- Add up migration script here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TYPE action_type AS ENUM ('verify-account', 'reset-password');

CREATE TABLE IF NOT EXISTS user_action_tokens (
      id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
      user_id UUID NOT NULL,
      token VARCHAR(32),
      action_type action_type NOT NULL,
      used_at TIMESTAMPTZ,
      expires_at TIMESTAMPTZ,
      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
      updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
      FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
      CONSTRAINT unique_user_action_type UNIQUE (user_id, action_type)
);