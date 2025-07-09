-- Add down migration script here

DROP TABLE IF EXISTS role_permissions;
DROP EXTENSION IF EXISTS "uuid-ossp";