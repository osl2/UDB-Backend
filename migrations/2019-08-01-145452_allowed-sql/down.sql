-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS subtasks;
CREATE TABLE subtasks (
    id CHAR(36) PRIMARY KEY NOT NULL,
    instruction TEXT NOT NULL,
    is_solution_verifiable BOOLEAN NOT NULL DEFAULT 'f',
    is_solution_visible BOOLEAN NOT NULL DEFAULT 'f',
    content TEXT -- data specific to subtask as JSON-Object
);
