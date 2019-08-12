pub mod entity;
pub mod import;
pub mod index;
pub mod schema;

pub use diesel::r2d2::PoolError;
pub use diesel::result::Error as UnderlyingError;

use diesel::{r2d2, PgConnection};

use std::fmt;
use std::sync::Once;

use crate::settings;

/// Database connection pool
type ConnectionPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

/// Represents an error that may happen on querying db
#[derive(Debug)]
pub enum QueryError {
    /// Failed to acquire db connection from connection pool
    PoolFailed(PoolError),
    /// Failed to perform db query
    QueryFailed(UnderlyingError),
}

pub fn connection_pool() -> ConnectionPool {
    static mut SHARED: *const ConnectionPool = std::ptr::null();
    static ONCE: Once = Once::new();

    unsafe {
        ONCE.call_once(|| {
            let pool = new_connection_pool(settings::shared().db())
                .expect("failed to escablish db connection");
            SHARED = Box::into_raw(Box::new(pool));
        });

        (*SHARED).clone()
    }
}

pub fn new_connection_pool(settings: &settings::Db) -> Result<ConnectionPool, PoolError> {
    let manager = r2d2::ConnectionManager::<PgConnection>::new(settings.url());
    r2d2::Builder::new()
        .max_size(settings.max_connections())
        .connection_timeout(settings.connection_timeout())
        .build(manager)
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
