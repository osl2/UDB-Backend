-- This file should undo anything in `up.sql`
CREATE UNIQUE INDEX databases_name_key ON public.databases USING btree (name);
ALTER TABLE databases ADD CONSTRAINT databases_name_key UNIQUE USING INDEX databases_name_key;