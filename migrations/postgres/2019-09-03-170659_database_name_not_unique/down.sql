-- This file should undo anything in `up.sql`
ALTER TABLE databases ADD CONSTRAINT databases_name_key UNIQUE (name);
