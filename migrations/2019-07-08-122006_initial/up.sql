-- Your SQL goes here
CREATE TABLE users (
    id CHAR(36) PRIMARY KEY NOT NULL,  -- CHAR(36) => UUID
    name TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    salt TEXT NOT NULL
);

-- Specifies if a user has access to a resource
CREATE TABLE access (
    user_id CHAR(36) NOT NULL,
    object_id CHAR(36) NOT NULL,
    PRIMARY KEY(user_id, object_id)
);

CREATE TABLE databases (
    id CHAR(36) PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    content TEXT NOT NULL
);

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
    instruction TEXT,
    is_solution_verifiable BOOLEAN,
    content BLOB, -- data specific to subtask as JSON-Object
    task_id CHAR(36),
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);
