create table index_files
(
    id serial not null,
    hash text not null,
    pending boolean default true not null,
    created_at timestamptz default now() not null,
    updated_at timestamptz default now() not null
);

create unique index index_files_hash_uindex
    on index_files (hash);

create unique index index_files_id_uindex
    on index_files (id);

alter table index_files
    add constraint index_files_pk
        primary key (id);

SELECT diesel_manage_updated_at('index_files');
