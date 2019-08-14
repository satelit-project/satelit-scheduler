pub mod entity;
pub mod import;
pub mod index;
pub mod schema;

pub use diesel::r2d2::PoolError;
pub use diesel::result::Error as UnderlyingError;

use diesel::{r2d2, PgConnection};
use lazy_static::lazy_static;

use std::fmt;

use crate::settings;

lazy_static! {
    static ref SHARED_POOL: ConnectionPool = {
        new_connection_pool(settings::shared().db()).expect("failed to escablish db connection")
    };
}

/// PostgresQL connection from connection pool
pub type PgPooledConnection = r2d2::PooledConnection<r2d2::ConnectionManager<PgConnection>>;

/// Database connection pool
#[derive(Clone)]
pub struct ConnectionPool(r2d2::Pool<r2d2::ConnectionManager<PgConnection>>);

/// Represents an error that may happen on querying db
#[derive(Debug)]
pub enum QueryError {
    /// Failed to acquire db connection from connection pool
    PoolFailed(PoolError),
    /// Failed to perform db query
    QueryFailed(UnderlyingError),
}

pub fn connection_pool() -> ConnectionPool {
    SHARED_POOL.clone()
}

pub fn new_connection_pool(settings: &settings::Db) -> Result<ConnectionPool, PoolError> {
    let manager = r2d2::ConnectionManager::<PgConnection>::new(settings.url());
    let pool = r2d2::Builder::new()
        .max_size(settings.max_connections())
        .connection_timeout(settings.connection_timeout())
        .build(manager)?;

    Ok(ConnectionPool(pool))
}

impl ConnectionPool {
    pub fn get(&self) -> Result<PgPooledConnection, PoolError> {
        self.0.get()
    }
}

impl std::fmt::Debug for ConnectionPool {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "pg connection pool")
    }
}

impl From<PoolError> for QueryError {
    fn from(e: PoolError) -> Self {
        QueryError::PoolFailed(e)
    }
}

impl From<UnderlyingError> for QueryError {
    fn from(e: UnderlyingError) -> Self {
        QueryError::QueryFailed(e)
    }
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use QueryError::*;

        match *self {
            PoolFailed(ref e) => <PoolError as fmt::Display>::fmt(&e, f),
            QueryFailed(ref e) => <UnderlyingError as fmt::Display>::fmt(&e, f),
        }
    }
}

impl std::error::Error for QueryError {}
