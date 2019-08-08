-- Your SQL goes here
CREATE TABLE aliases (
    alias TEXT PRIMARY KEY NOT NULL,
    object_id CHAR(36) NOT NULL,
    object_type INTEGER
);