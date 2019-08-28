-- Your SQL goes here
PRAGMA foreign_keys=OFF;
CREATE TABLE new_tasks (
   id CHAR(36) PRIMARY KEY NOT NULL,
   database_id CHAR(36),
   name TEXT,
   FOREIGN KEY (database_id) REFERENCES databases(id)
);
INSERT INTO new_tasks (id, database_id, name) SELECT id, database_id, name FROM tasks;
DROP TABLE tasks;
ALTER TABLE new_tasks RENAME TO tasks;
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;