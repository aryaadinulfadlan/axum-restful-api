-- Add down migration script here

DROP TABLE IF EXISTS roles;
DROP EXTENSION IF EXISTS "uuid-ossp";
DROP TYPE IF EXISTS role_type;