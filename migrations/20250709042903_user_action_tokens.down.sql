-- Add down migration script here

DROP TABLE IF EXISTS user_action_tokens;
DROP EXTENSION IF EXISTS "uuid-ossp";
DROP TYPE IF EXISTS action_type;