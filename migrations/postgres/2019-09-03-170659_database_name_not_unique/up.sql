-- Your SQL goes here
ALTER TABLE databases DROP CONSTRAINT databases_name_key;
DROP INDEX IF EXISTS databases_name_key;