mod convert;

use chrono::{DateTime, Utc};
use diesel::prelude::*;

use crate::db::schema::{index_files, failed_imports};

#[derive(Debug, Clone, Copy, FromSqlRow, AsExpression)]
pub enum Source {
    Anidb = 0
}

#[derive(Queryable, Identifiable)]
pub struct IndexFile {
    pub id: i32,
    pub source: Source,
    pub hash: String,
    pub pending: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable, Identifiable)]
pub struct FailedImport {
    pub id: i32,
    pub index_id: i32,
    pub title_ids: Vec<i32>,
    pub reimported: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
