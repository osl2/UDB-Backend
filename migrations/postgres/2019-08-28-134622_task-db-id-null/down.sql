-- This file should undo anything in `up.sql`
DELETE FROM tasks WHERE database_id IS NULL;
ALTER TABLE tasks ALTER COLUMN database_id SET NOT NULL;