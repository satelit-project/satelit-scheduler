use futures::future::*;
use tower_grpc::client;
use tower_grpc::generic::client::GrpcService;
use tower_grpc::Request;
use uuid::Uuid;

use std::marker::PhantomData;

use super::StateError;
use crate::block::blocking;
use crate::db::entity::{self, FailedImport, IndexFile};
use crate::db::import::FailedImports;
use crate::db::index::IndexFiles;
use crate::db::QueryError;
use crate::proto::data;
use crate::proto::import::client::ImportService;
use crate::proto::import::ImportIntent;

#[derive(Debug)]
pub struct ImportIndex<T, R> {
    client: ImportService<T>,
    index_files: IndexFiles,
    failed_imports: FailedImports,
    _request: PhantomData<R>,
}

#[derive(Clone)]
struct DbContext {
    index_file: IndexFile,
    index_files: IndexFiles,
    failed_import: Option<FailedImport>,
    failed_imports: FailedImports,
}

impl<T, R> ImportIndex<T, R> {
    pub fn new(
        client: ImportService<T>,
        index_files: IndexFiles,
        failed_imports: FailedImports,
    ) -> Self {
        ImportIndex {
            client,
            index_files,
            failed_imports,
            _request: PhantomData,
        }
    }
}

impl<T, R> ImportIndex<T, R>
where
    T: GrpcService<R> + Send,
    T::Future: Send,
    T::ResponseBody: Send,
    <<T as GrpcService<R>>::ResponseBody as tower_grpc::Body>::Data: Send,
    R: Send,
    client::unary::Once<ImportIntent>: client::Encodable<R>,
{
    pub fn import(
        self,
        index_file: IndexFile,
    ) -> impl Future<Item = Self, Error = StateError> + Send {
        // TODO: don't want to refactor yet, waiting for async/await beta

        let source = entity::Source::Anidb;
        let ImportIndex {
            client,
            index_files,
            failed_imports,
            ..
        } = self;

        // retrieve ids to reimport
        blocking(move || {
            let failed_import = failed_imports.with_source(source)?;
            let context = DbContext {
                index_file,
                index_files,
                failed_import,
                failed_imports,
            };
            Ok(context)
        })
        .from_err()
        // wait until grpc client ready to send requests
        .join(client.ready().from_err())
        // send import intent (start index import)
        .and_then(move |(context, mut client)| {
            let source = <entity::Source as Into<data::Source>>::into(source) as i32;
            let mut reimport_ids = vec![];
            if let Some(ref failed_import) = context.failed_import {
                reimport_ids.extend(failed_import.title_ids.iter())
            }

            let intent = ImportIntent {
                id: Uuid::new_v4().to_string(),
                source,
                dump_url: context.index_file.url.clone(),
                reimport_ids,
            };

            client
                .start_import(Request::new(intent))
                .from_err()
                .join(Ok(context))
                .and_then(move |data| Ok((client, data)))
        })
        // on response
        .and_then(move |(client, (response, context))| {
            let result = response.into_inner();

            // preparing to write data from response to DB
            blocking(move || {
                let DbContext {
                    index_files,
                    index_file,
                    failed_imports,
                    failed_import,
                } = context;

                // mark previously failed to import ids as reimported
                if let Some(failed_import) = failed_import {
                    failed_imports.mark_reimported(failed_import)?;
                }

                // remember any titles that failed to import
                if !result.skipped_ids.is_empty() {
                    failed_imports.create(&index_file, &result.skipped_ids)?;
                }

                // mark index file as imported
                index_files.mark_processed(index_file)?;
                Ok((index_files, failed_imports))
            })
            .from_err()
            // recreate myself to keep client
            .and_then(move |(index_files, failed_imports)| {
                let me = Self::new(client, index_files, failed_imports);
                Ok(me)
            })
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
