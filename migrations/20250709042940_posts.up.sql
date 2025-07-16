-- Add up migration script here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS posts (
     id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
     user_id UUID NOT NULL,
     title VARCHAR(20) NOT NULL,
     content TEXT NOT NULL,
     tags VARCHAR(20)[] NOT NULL,
     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
     updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
     FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

INSERT INTO posts (id, user_id, title, content, tags)
VALUES
    (
        'db5d4c6c-15d5-4555-89ad-555c7f0e4cf9',
        '50b466de-2de4-4e40-bdec-08270f23a8c8',
        'Clark First Post',
        'The first post by Clark Kent',
        '{clark-kent, first}'
    ),
    (
        'fc892bcd-bada-4273-94f5-fdd11ae8fd25',
        '50b466de-2de4-4e40-bdec-08270f23a8c8',
        'Clark Second Post',
        'The second post by Clark Kent',
        '{clark-kent, second}'
    ),
    (
        '8529e9cb-b30e-4f74-aed8-7469bb2de48c',
        '4c176d2e-787f-44c0-a578-c5c6d15503bb',
        'Bruce First Post',
        'The first post by Bruce Wayne',
        '{bruce-wayne, first}'
    ),
    (
        '203fb9dd-8f28-470d-a16e-715b9f9e2468',
        '4c176d2e-787f-44c0-a578-c5c6d15503bb',
        'Bruce Second Post',
        'The second post by Bruce Wayne',
        '{bruce-wayne, second}'
    ),
    (
        '07112266-83d1-408f-bd19-ec4b80f760a0',
        'e1b4e485-fa48-4d59-8758-e7f988d5cc17',
        'Princess First Post',
        'The first post by Princess Diana',
        '{princess-diana, first}'
    ),
    (
        '717cae2a-08dc-4db9-81e9-5544836674fc',
        'e1b4e485-fa48-4d59-8758-e7f988d5cc17',
        'Princess Second Post',
        'The second post by Princess Diana',
        '{princess-diana, second}'
    );