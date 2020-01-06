mod convert;

use chrono::{DateTime, Utc};
use diesel::sql_types::Integer;

use crate::{
    db::schema::{failed_imports, index_files},
    proto::uuid::Uuid,
};

/// Represents anime entry location in external database.
#[repr(C)]
#[sql_type = "Integer"]
#[derive(Debug, Clone, Copy, FromSqlRow, AsExpression)]
pub enum Source {
    Anidb = 0,
}

/// Represents an index file of all anime entries in external database.
#[derive(Clone, Queryable, Identifiable)]
pub struct IndexFile {
    pub id: Uuid,
    pub source: Source,
    pub hash: String,
    pub pending: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Represents list of failed anime imports for an index file.
#[derive(Clone, Queryable, Identifiable)]
pub struct FailedImport {
    pub id: Uuid,
    pub index_id: Uuid,
    pub title_ids: Vec<i32>,
    pub reimported: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
