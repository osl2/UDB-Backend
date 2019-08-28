-- This file should undo anything in `up.sql`
CREATE TABLE new_tasks (
   id CHAR(36) PRIMARY KEY NOT NULL,
   name TEXT,
   database_id CHAR(36) NOT NULL,
   FOREIGN KEY (database_id) REFERENCES databases(id)
);
INSERT INTO new_tasks SELECT tasks.id, tasks.name, tasks.database_id FROM tasks;
DROP TABLE tasks;
ALTER TABLE new_tasks RENAME TO tasks;