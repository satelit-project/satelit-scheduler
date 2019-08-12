use diesel::prelude::*;

use crate::db::entity::{FailedImport, IndexFile, Source};
use crate::db::{ConnectionPool, QueryError};

#[derive(Clone)]
pub struct FailedImports {
    pool: ConnectionPool,
}

impl FailedImports {
    pub fn new(pool: ConnectionPool) -> Self {
        FailedImports { pool }
    }

    pub fn create(&self, index: &IndexFile, ids: &[i32]) -> Result<FailedImport, QueryError> {
        use crate::db::schema::failed_imports::dsl::*;

        let conn = self.pool.get()?;
        let value = diesel::insert_into(failed_imports)
            .values((index_id.eq(index.id), title_ids.eq(ids)))
            .get_result(&conn)?;

        Ok(value)
    }

    pub fn with_source(&self, src: Source) -> Result<Option<FailedImport>, QueryError> {
        use crate::db::schema::failed_imports;
        use crate::db::schema::index_files;

        let conn = self.pool.get()?;
        let value = failed_imports::table
            .inner_join(index_files::table)
            .filter(failed_imports::reimported.eq(false))
            .filter(index_files::source.eq(src as i32))
            .order(failed_imports::created_at.desc())
            .select(failed_imports::all_columns)
            .first::<FailedImport>(&conn)
            .optional()?;

        Ok(value)
    }

    pub fn mark_reimported(&self, failed: FailedImport) -> Result<FailedImport, QueryError> {
        use crate::db::schema::failed_imports::dsl::*;

        let conn = self.pool.get()?;
        let value = diesel::update(&failed)
            .set(reimported.eq(true))
            .get_result(&conn)?;

        Ok(value)
    }
}
