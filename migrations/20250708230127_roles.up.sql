-- Add up migration script here

CREATE TYPE role_type AS ENUM ('admin', 'user');
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS roles (
     id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
     name role_type NOT NULL DEFAULT 'user',
     description VARCHAR(100) NOT NULL,
     created_at TIMESTAMPTZ DEFAULT NOW(),
     updated_at TIMESTAMPTZ DEFAULT NOW()
);

INSERT INTO roles (id, name, description)
VALUES
    ('e3488ac6-7012-4d95-a002-663b9a6f879a', 'user', 'Can use basic features for their own account.'),
    ('4b30ed16-06bc-4f7f-8293-6cb8a040267e', 'admin', 'Has full access to manage users, content, and system settings.');