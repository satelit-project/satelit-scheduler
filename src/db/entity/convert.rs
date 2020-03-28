use std::{convert::TryFrom, io::Write};

use diesel::{
    backend::Backend,
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, Output, ToSql},
    sql_types::{Integer, Uuid},
};

use super::Source;
use crate::proto::uuid;

impl<DB> FromSql<Integer, DB> for Source
where
    DB: Backend,
    i32: FromSql<Integer, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        match i32::from_sql(bytes)? {
            1 => Ok(Source::Anidb),
            x => Err(format!("Unrecognized Source case: {}", x).into()),
        }
    }
}

impl<DB> ToSql<Integer, DB> for Source
where
    DB: Backend,
    i32: ToSql<Integer, DB>,
{
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        (*self as i32).to_sql(out)
    }
}

// MARK: impl Uuid

#[derive(FromSqlRow, AsExpression)]
#[diesel(foreign_derive)]
#[sql_type = "Uuid"]
#[allow(dead_code)]
struct UuidProxy(uuid::Uuid);

impl FromSql<Uuid, Pg> for uuid::Uuid {
    fn from_sql(bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
        // assuming that db always has correctly encoded uuid
        let bytes = not_none!(bytes);
        uuid::Uuid::try_from(bytes).map_err(Into::into)
    }
}

impl ToSql<Uuid, Pg> for uuid::Uuid {
    fn to_sql<W: Write>(
        &self,
        out: &mut diesel::serialize::Output<W, Pg>,
    ) -> diesel::serialize::Result {
        let bytes = self.as_slice();
        if bytes.is_empty() {
            return Ok(diesel::serialize::IsNull::Yes);
        }

        out.write_all(bytes)
            .map(|_| diesel::serialize::IsNull::No)
            .map_err(Into::into)
    }
}
