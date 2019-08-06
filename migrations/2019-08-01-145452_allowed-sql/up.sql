-- Your SQL goes here
DROP TABLE IF EXISTS subtasks;
CREATE TABLE subtasks (
    id CHAR(36) PRIMARY KEY NOT NULL,
    instruction TEXT NOT NULL,
    is_solution_verifiable BOOLEAN NOT NULL DEFAULT 'f',
    is_solution_visible BOOLEAN NOT NULL DEFAULT 'f',
    allowed_sql INTEGER NOT NULL DEFAULT 0,  -- 0: All statements allowed, 1: Only select-statements
    content TEXT -- data specific to subtask as JSON-Object
);