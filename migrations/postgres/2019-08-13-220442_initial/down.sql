-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS access;
DROP TABLE IF EXISTS databases;
DROP TABLE IF EXISTS courses;
DROP TABLE IF EXISTS worksheets;
DROP TABLE IF EXISTS tasks;
DROP TABLE IF EXISTS subtasks;
DROP TABLE IF EXISTS worksheets_in_courses;
DROP TABLE IF EXISTS tasks_in_worksheets;
DROP TABLE IF EXISTS subtasks_in_tasks;
DROP TABLE IF EXISTS aliases;