-- Add up migration script here

CREATE EXTENSION IF NOT EXISTS CITEXT;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS users (
     id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
     role_id UUID NOT NULL,
     name VARCHAR(50) NOT NULL,
     email CITEXT UNIQUE NOT NULL,
     password TEXT NOT NULL,
     is_verified BOOLEAN NOT NULL DEFAULT FALSE,
     created_at TIMESTAMPTZ DEFAULT NOW(),
     updated_at TIMESTAMPTZ DEFAULT NOW(),
     FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
);
CREATE INDEX users_email_idx ON users (email);

INSERT INTO users (id, role_id, name, email, password, is_verified)
VALUES
    (
        '50b466de-2de4-4e40-bdec-08270f23a8c8',
        'e3488ac6-7012-4d95-a002-663b9a6f879a',
        'Clark Kent',
        'clark_kent@gmail.com',
        '$2a$10$Ex86AamJanKW8yuwxCVYme22uA8zpOIvEpjpNGGeMIxxMj1r98GgO',
        true
    ),
    (
        '4c176d2e-787f-44c0-a578-c5c6d15503bb',
        'e3488ac6-7012-4d95-a002-663b9a6f879a',
        'Bruce Wayne',
        'bruce_wayne@gmail.com',
        '$2a$10$UWWHQC1SyMmOGGDcHM1We.gjHN.vP79vgH3jO22XaACznioNxy2D.',
        true
    ),
    (
        'e1b4e485-fa48-4d59-8758-e7f988d5cc17',
        '4b30ed16-06bc-4f7f-8293-6cb8a040267e',
        'Princess Diana',
        'princess_diana@gmail.com',
        '$2a$10$PnrNtmnRS.Ap6bQSPqCS5eKTGGFGYVHJ73si8G2Ta240sbdZEU8Ke',
        true
    );