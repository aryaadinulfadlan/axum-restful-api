-- Add down migration script here

DROP TABLE IF EXISTS comments;
DROP EXTENSION IF EXISTS "uuid-ossp";