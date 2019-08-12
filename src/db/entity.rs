mod convert;

use crate::db::schema::{failed_imports, index_files};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, FromSqlRow, AsExpression)]
pub enum Source {
    Anidb = 0,
}

#[derive(Clone, Queryable, Identifiable)]
pub struct IndexFile {
    pub id: i32,
    pub source: Source,
    pub hash: String,
    pub pending: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Queryable, Identifiable)]
pub struct FailedImport {
    pub id: i32,
    pub index_id: i32,
    pub title_ids: Vec<i32>,
    pub reimported: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
