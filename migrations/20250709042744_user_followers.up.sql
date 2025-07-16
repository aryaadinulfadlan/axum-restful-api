-- Add up migration script here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS user_followers (
      follower_id UUID NOT NULL,
      following_id UUID NOT NULL,
      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
      updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
      PRIMARY KEY (follower_id, following_id),
      FOREIGN KEY (follower_id) REFERENCES users(id) ON DELETE CASCADE,
      FOREIGN KEY (following_id) REFERENCES users(id) ON DELETE CASCADE
);

INSERT INTO user_followers (follower_id, following_id)
VALUES
    ('50b466de-2de4-4e40-bdec-08270f23a8c8', 'e1b4e485-fa48-4d59-8758-e7f988d5cc17'),
    ('4c176d2e-787f-44c0-a578-c5c6d15503bb', 'e1b4e485-fa48-4d59-8758-e7f988d5cc17'),
    ('50b466de-2de4-4e40-bdec-08270f23a8c8', '4c176d2e-787f-44c0-a578-c5c6d15503bb'),
    ('4c176d2e-787f-44c0-a578-c5c6d15503bb', '50b466de-2de4-4e40-bdec-08270f23a8c8');