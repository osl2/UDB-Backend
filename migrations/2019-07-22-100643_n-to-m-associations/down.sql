-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS courses;
DROP TABLE IF EXISTS worksheets;
DROP TABLE IF EXISTS tasks;
DROP TABLE IF EXISTS subtasks;
DROP TABLE IF EXISTS worksheets_in_courses;
DROP TABLE IF EXISTS tasks_in_worksheets;
DROP TABLE IF EXISTs subtasks_in_tasks;

CREATE TABLE courses (
    id CHAR(36) PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT
);

CREATE TABLE worksheets (
    id CHAR(36) PRIMARY KEY NOT NULL,
    name TEXT,
    is_online BOOLEAN NOT NULL DEFAULT 'f',
    is_solution_online BOOLEAN NOT NULL DEFAULT 'f',
    course_id CHAR(36),
    FOREIGN KEY (course_id) REFERENCES courses(id)
);

CREATE TABLE tasks (
    id CHAR(36) PRIMARY KEY NOT NULL,
    worksheet_id CHAR(36),
    FOREIGN KEY (worksheet_id) REFERENCES worksheets(id)
);

CREATE TABLE subtasks (
    id CHAR(36) PRIMARY KEY NOT NULL,
    instruction TEXT NOT NULL,
    is_solution_verifiable BOOLEAN NOT NULL DEFAULT 'f',
    is_solution_visible BOOLEAN NOT NULL DEFAULT 'f',
    content TEXT, -- data specific to subtask as JSON-Object
    task_id CHAR(36),
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);
