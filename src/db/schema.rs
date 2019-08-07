table! {
    index_files (id) {
        id -> Int4,
        hash -> Text,
        pending -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
