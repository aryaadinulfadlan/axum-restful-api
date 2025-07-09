-- Add down migration script here

DROP TABLE IF EXISTS user_followers;
DROP EXTENSION IF EXISTS "uuid-ossp";