pub mod import;
pub mod index;
pub mod scrape;

use crate::block::BlockingError;
use crate::db::QueryError;
use reqwest::Error as HttpError;
use tower_grpc::Status;

#[derive(Debug)]
pub enum StateError {
    StorageError(QueryError),
    GrpcError(Status),
    HttpError(HttpError),
    UnknownError(Box<dyn std::error::Error>),
}

pub trait State {}

impl From<Status> for StateError {
    fn from(e: Status) -> Self {
        StateError::GrpcError(e)
    }
}

impl From<QueryError> for StateError {
    fn from(e: QueryError) -> Self {
        StateError::StorageError(e)
    }
}

impl From<HttpError> for StateError {
    fn from(e: HttpError) -> Self {
        StateError::HttpError(e)
    }
}

impl From<BlockingError<QueryError>> for StateError {
    fn from(e: BlockingError<QueryError>) -> Self {
        match e {
            BlockingError::Error(e) => e.into(),
            e => StateError::UnknownError(Box::new(e)),
        }
    }
}
