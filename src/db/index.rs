use diesel::prelude::*;

use crate::db::{
    entity::{IndexFile, Source},
    ConnectionPool, QueryError,
};

#[derive(Debug, Clone)]
pub struct IndexFiles {
    pool: ConnectionPool,
}

impl IndexFiles {
    pub fn new(pool: ConnectionPool) -> Self {
        IndexFiles { pool }
    }

    pub fn queue(&self, new_hash: &str, src: Source) -> Result<IndexFile, QueryError> {
        use crate::db::schema::index_files::dsl::*;

        let conn = self.pool.get()?;
        let index = diesel::insert_into(index_files)
            .values((hash.eq(new_hash), source.eq(src)))
            .on_conflict_do_nothing()
            .get_result(&conn)?;

        Ok(index)
    }

    pub fn latest_processed(&self, latest: &IndexFile) -> Result<Option<IndexFile>, QueryError> {
        use crate::db::schema::index_files::dsl::*;

        if !latest.pending {
            return Ok(Some(latest.clone()));
        }

        let conn = self.pool.get()?;
        let index: Result<IndexFile, _> = index_files
            .select(index_files::all_columns())
            .filter(pending.eq(false))
            .order_by(updated_at.desc())
            .first(&conn);

        match index {
            Ok(index) => Ok(Some(index)),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn mark_processed(&self, index_file: IndexFile) -> Result<IndexFile, QueryError> {
        use crate::db::schema::index_files::dsl::*;

        let conn = self.pool.get()?;
        let new_index = diesel::update(index_files.find(index_file.id))
            .set(pending.eq(false))
            .get_result(&conn)?;

        Ok(new_index)
    }
}
