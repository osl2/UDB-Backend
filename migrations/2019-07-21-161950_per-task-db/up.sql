-- Your SQL goes here
DROP TABLE IF EXISTS tasks;
CREATE TABLE tasks (
    id CHAR(36) PRIMARY KEY NOT NULL,
    database_id CHAR(36) NOT NULL,
    FOREIGN KEY (database_id) REFERENCES databases(id)
);