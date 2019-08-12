use diesel::prelude::*;

use crate::db::entity::IndexFile;
use crate::db::{ConnectionPool, QueryError};

#[derive(Clone)]
pub struct IndexFiles {
    pool: ConnectionPool,
}

impl IndexFiles {
    pub fn queue(&self, new_hash: &str) -> Result<IndexFile, QueryError> {
        use crate::db::schema::index_files::dsl::*;

        let conn = self.pool.get()?;

        let index = diesel::insert_into(index_files)
            .values(hash.eq(new_hash))
            .on_conflict_do_nothing()
            .get_result(&conn)?;

        Ok(index)
    }
}