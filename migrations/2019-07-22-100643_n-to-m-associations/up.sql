-- Your SQL goes here
DROP TABLE IF EXISTS courses;
DROP TABLE IF EXISTS worksheets;
DROP TABLE IF EXISTS tasks;
DROP TABLE IF EXISTS subtasks;

CREATE TABLE courses (
    id CHAR(36) PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT
);

CREATE TABLE worksheets (
    id CHAR(36) PRIMARY KEY NOT NULL,
    name TEXT,
    is_online BOOLEAN NOT NULL DEFAULT 'f',
    is_solution_online BOOLEAN NOT NULL DEFAULT 'f'
);

CREATE TABLE tasks (
    id CHAR(36) PRIMARY KEY NOT NULL,
    database_id CHAR(36) NOT NULL,
    FOREIGN KEY (database_id) REFERENCES databases(id)
);

CREATE TABLE subtasks (
    id CHAR(36) PRIMARY KEY NOT NULL,
    instruction TEXT NOT NULL,
    is_solution_verifiable BOOLEAN NOT NULL DEFAULT 'f',
    is_solution_visible BOOLEAN NOT NULL DEFAULT 'f',
    content TEXT -- data specific to subtask as JSON-Object
);

CREATE TABLE worksheets_in_courses (
    worksheet_id CHAR(36) NOT NULL,
    course_id CHAR(36) NOT NULL,
    position INTEGER,
    FOREIGN KEY (worksheet_id) REFERENCES worksheets(id),
    FOREIGN KEY (course_id) REFERENCES courses(id),
    PRIMARY KEY (worksheet_id, course_id)
);

CREATE TABLE tasks_in_worksheets (
    task_id CHAR(36) NOT NULL,
    worksheet_id CHAR(36) NOT NULL,
    position INTEGER,
    FOREIGN KEY (task_id) REFERENCES tasks(id),
    FOREIGN KEY (worksheet_id) REFERENCES worksheets(id),
    PRIMARY KEY (task_id, worksheet_id)
);

CREATE TABLE subtasks_in_tasks (
    subtask_id CHAR(36) NOT NULL,
    task_id CHAR(36) NOT NULL,
    position INTEGER NOT NULL,
    FOREIGN KEY (subtask_id) REFERENCES subtasks(id),
    FOREIGN KEY (task_id) REFERENCES tasks(id),
    PRIMARY KEY (subtask_id, task_id)
);