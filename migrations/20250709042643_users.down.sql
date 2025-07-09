-- Add down migration script here

DROP TABLE IF EXISTS users;
DROP EXTENSION IF EXISTS "uuid-ossp";
DROP EXTENSION IF EXISTS CITEXT;
DROP INDEX IF EXISTS users_email_idx;