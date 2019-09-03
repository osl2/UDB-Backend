-- This file should undo anything in `up.sql`
ALTER TABLE databases ADD CONSTRAINT unique_name UNIQUE (name);