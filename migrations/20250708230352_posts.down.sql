-- Add down migration script here

DROP TABLE IF EXISTS posts;
DROP EXTENSION IF EXISTS "uuid-ossp";