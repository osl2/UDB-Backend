-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS tasks;
CREATE TABLE tasks (
    id CHAR(36) PRIMARY KEY NOT NULL,
    worksheet_id CHAR(36),
    FOREIGN KEY (worksheet_id) REFERENCES worksheets(id)
);