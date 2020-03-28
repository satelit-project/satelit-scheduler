table! {
    failed_imports (id) {
        id -> Uuid,
        index_id -> Uuid,
        title_ids -> Array<Int4>,
        reimported -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    index_files (id) {
        id -> Uuid,
        source -> Int4,
        file_path -> Text,
        pending -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

joinable!(failed_imports -> index_files (index_id));

allow_tables_to_appear_in_same_query!(
    failed_imports,
    index_files,
);
