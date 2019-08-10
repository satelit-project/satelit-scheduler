table! {
    failed_imports (id) {
        id -> Int4,
        index_id -> Int4,
        title_ids -> Array<Int4>,
        reimported -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    index_files (id) {
        id -> Int4,
        source -> Int4,
        hash -> Text,
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
