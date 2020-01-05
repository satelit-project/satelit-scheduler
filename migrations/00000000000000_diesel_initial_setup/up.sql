-- This file was automatically created by Diesel to setup helper functions
-- and other internal bookkeeping. This file is safe to edit, any future
-- changes will be added to existing projects as new migrations.


create extension if not exists "uuid-ossp";


-- Sets up a trigger for the given table to automatically set a column called
-- `updated_at` whenever the row is modified (unless `updated_at` was included
-- in the modified columns)
--
-- # Example
--
-- ```sql
-- CREATE TABLE users (id SERIAL PRIMARY KEY, updated_at TIMESTAMP NOT NULL DEFAULT NOW());
--
-- SELECT diesel_manage_updated_at('users');
-- ```
create or replace function diesel_manage_updated_at(_tbl regclass) returns void as $$
begin
    execute format('create trigger set_updated_at before update on %s
                    for each row execute procedure diesel_set_updated_at()', _tbl);
end;
$$ language plpgsql;

create or replace function diesel_set_updated_at() returns trigger as $$
begin
    if (
        new is distinct from old and
        new.updated_at is not distinct from old.updated_at
    ) then
        new.updated_at := current_timestamp;
    end if;
    return new;
end;
$$ language plpgsql;
