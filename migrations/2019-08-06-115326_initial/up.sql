-- index_files --

create table index_files
(
    id         uuid        default uuid_generate_v4() not null,
    source     int                                    not null,
    file_path  text                                   not null,
    pending    boolean     default true               not null,
    created_at timestamptz default now()              not null,
    updated_at timestamptz default now()              not null
);

create unique index index_files_file_path_uindex
    on index_files (file_path);

create unique index index_files_id_uindex
    on index_files (id);

alter table index_files
    add constraint index_files_pk
        primary key (id);

SELECT diesel_manage_updated_at('index_files');

-- failed_imports --

create table failed_imports
(
    id         uuid        default uuid_generate_v4() not null,
    index_id   uuid                                   not null
        constraint failed_imports_index_files_id_fk
            references index_files
            on update cascade on delete cascade,
    title_ids  int[]                                  not null,
    reimported bool        default false              not null,
    created_at timestamptz default now()              not null,
    updated_at timestamptz default now()              not null
);

create unique index failed_imports_id_uindex
    on failed_imports (id);

alter table failed_imports
    add constraint failed_imports_pk
        primary key (id);

SELECT diesel_manage_updated_at('failed_imports')
