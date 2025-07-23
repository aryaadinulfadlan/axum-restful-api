-- Add up migration script here

CREATE TABLE IF NOT EXISTS refresh_tokens (
    user_id UUID NOT NULL PRIMARY KEY,
    token TEXT NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);