use futures::future::*;
use tower_grpc::client;
use tower_grpc::generic::client::GrpcService;
use tower_grpc::Request;
use uuid::Uuid;

use std::error::Error;
use std::marker::PhantomData;

use crate::block::{blocking, BlockingError};
use crate::db::entity;
use crate::db::import::FailedImports;
use crate::db::QueryError;
use crate::proto::data;
use crate::proto::import::client::ImportService;
use crate::proto::import::ImportIntent;
use crate::settings;

#[derive(Debug)]
pub enum ImportError {
    StorageError(Box<dyn Error>),
    NetworkError(Box<dyn Error>),
}

pub struct ImportIndex<T, R> {
    client: ImportService<T>,
    index_file: entity::IndexFile,
    failed_imports: FailedImports,
    _phantom: PhantomData<R>,
}

struct DbContext {
    index_file: entity::IndexFile,
    failed_import: Option<entity::FailedImport>,
    failed_imports: FailedImports,
}

impl<T, R> ImportIndex<T, R>
where
    T: GrpcService<R>,
    client::unary::Once<ImportIntent>: client::Encodable<R>,
{
    fn import(self) -> impl Future<Item = (), Error = ImportError> {
        // TODO: don't want to refactor yet, waiting for async/await beta

        let source = entity::Source::Anidb;
        let ImportIndex {
            client,
            index_file,
            failed_imports,
            ..
        } = self;

        // retrieve ids to reimport
        blocking(move || {
            let failed_import = failed_imports.with_source(source)?;
            let context = DbContext {
                index_file: index_file.clone(),
                failed_import,
                failed_imports: failed_imports.clone(),
            };
            Ok(context)
        })
        .map_err(|e: BlockingError<QueryError>| ImportError::StorageError(Box::new(e)))
        // wait until grpc client ready to send requests
        .join(
            client
                .ready()
                .map_err(|e| ImportError::NetworkError(Box::new(e))),
        )
        // send import intent (start index import)
        .and_then(move |(context, mut client)| {
            let source = <entity::Source as Into<data::Source>>::into(source) as i32;
            let dump_url = settings::shared().anidb().dump_url().to_string();
            let mut reimport_ids = vec![];
            if let Some(ref failed_import) = context.failed_import {
                reimport_ids.extend(failed_import.title_ids.iter())
            }

            let intent = ImportIntent {
                id: Uuid::new_v4().to_string(),
                source,
                dump_url,
                reimport_ids,
            };

            client
                .start_import(Request::new(intent))
                .map_err(|e| ImportError::NetworkError(Box::new(e)))
                .join(Ok(context))
        })
        // on response
        .and_then(move |(response, context)| {
            let result = response.into_inner();

            // preparing to write data from response to DB
            blocking(move || {
                // mark previously failed to import ids as reimported
                if let Some(ref failed_import) = context.failed_import {
                    context
                        .failed_imports
                        .mark_reimported(failed_import.clone())?;
                }

                // remember any titles that failed to import
                if !result.skipped_ids.is_empty() {
                    context
                        .failed_imports
                        .create(&context.index_file, &result.skipped_ids)?;
                }

                Ok(())
            })
            .map_err(|e: BlockingError<QueryError>| ImportError::StorageError(Box::new(e)))
        })
    }
}

impl From<entity::Source> for data::Source {
    fn from(s: entity::Source) -> Self {
        match s {
            entity::Source::Anidb => data::Source::Anidb,
        }
    }
}

impl From<QueryError> for tower_grpc::Status {
    fn from(e: QueryError) -> Self {
        use tower_grpc::{Code, Status};
        Status::new(Code::Internal, e.to_string())
    }
}
