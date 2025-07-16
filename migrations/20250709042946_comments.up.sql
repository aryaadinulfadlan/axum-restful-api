-- Add up migration script here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS comments (
    id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
    user_id UUID NOT NULL,
    post_id UUID NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE
);

INSERT INTO comments (id, user_id, post_id, content)
VALUES
    (
        '9e9c3497-73a4-48df-b6b9-6f8bd2c91cf2',
        '50b466de-2de4-4e40-bdec-08270f23a8c8',
        'db5d4c6c-15d5-4555-89ad-555c7f0e4cf9',
        'Clark Kent commented his own post'
    ),
    (
        'd7ef9b51-2e26-4dfc-b3b2-dbca52c48038',
        '4c176d2e-787f-44c0-a578-c5c6d15503bb',
        'db5d4c6c-15d5-4555-89ad-555c7f0e4cf9',
        'Bruce Wayne commented Clark post'
    ),
    (
        'e52ed4f4-6841-4e9d-a434-8fa8ad6288ea',
        'e1b4e485-fa48-4d59-8758-e7f988d5cc17',
        'db5d4c6c-15d5-4555-89ad-555c7f0e4cf9',
        'Princess Diana commented Clark post'
    ),
    (
        'a30e2f41-b9ed-44c7-bb4d-44d1651a292a',
        'e1b4e485-fa48-4d59-8758-e7f988d5cc17',
        '8529e9cb-b30e-4f74-aed8-7469bb2de48c',
        'Princess Diana commented Bruce post'
    ),
    (
        '882007bf-4f6c-4588-a40e-0bab6d05b0eb',
        '4c176d2e-787f-44c0-a578-c5c6d15503bb',
        '8529e9cb-b30e-4f74-aed8-7469bb2de48c',
        'Bruce Wayne commented his own post'
    ),
    (
        '659c0922-3c7c-42b6-acea-5ac04d8efe2a',
        'e1b4e485-fa48-4d59-8758-e7f988d5cc17',
        '07112266-83d1-408f-bd19-ec4b80f760a0',
        'Princess Diana commented her own post'
    );