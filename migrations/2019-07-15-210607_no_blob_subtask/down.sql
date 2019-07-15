-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS subtasks;
CREATE TABLE subtasks (
    id CHAR(36) PRIMARY KEY NOT NULL,
    instruction TEXT,
    is_solution_verifiable BOOLEAN,
    content BLOB, -- data specific to subtask as JSON-Object
    task_id CHAR(36),
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);