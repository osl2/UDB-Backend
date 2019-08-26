-- This file should undo anything in `up.sql`
PRAGMA foreign_keys=OFF;
BEGIN TRANSACTION;
CREATE TABLE old_tasks (
    id CHAR(36) PRIMARY KEY NOT NULL,
    database_id CHAR(36) NOT NULL,
    FOREIGN KEY (database_id) REFERENCES databases(id)
);
INSERT INTO old_tasks SELECT (tasks.id, tasks.database_id) FROM tasks;
DROP TABLE tasks;
ALTER TABLE old_tasks RENAME TO tasks;
PRAGMA foreign_key_check;
COMMIT TRANSACTION;
PRAGMA foreign_keys=ON;