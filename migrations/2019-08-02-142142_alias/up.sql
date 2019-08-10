-- Your SQL goes here
DROP TABLE IF EXISTS aliases;
CREATE TABLE aliases (
    alias TEXT PRIMARY KEY NOT NULL,
    object_id CHAR(36) NOT NULL,
    object_type INTEGER NOT NULL
);