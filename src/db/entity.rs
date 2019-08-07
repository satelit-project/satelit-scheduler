use chrono::{DateTime, Utc};

#[derive(Queryable)]
pub struct IndexFile {
    pub id: i32,
    pub hash: String,
    pub pending: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
