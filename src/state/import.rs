use tonic::transport::Channel;
use tokio::task;

use super::StateError;
use crate::db::entity::{self, FailedImport, IndexFile};
use crate::db::import::FailedImports;
use crate::db::index::IndexFiles;
use crate::proto::import::import_service_client::ImportServiceClient;
use crate::proto::import::{ImportIntent, ImportIntentResult};
use crate::proto::uuid::Uuid;
use crate::proto::data;

/// Ask import service to start importing new database index file.
pub struct ImportIndex<F> {
    /// RPC client for importing service.
    client: ImportServiceClient<Channel>,

    /// Database access layer for all index files.
    index_files: IndexFiles,

    /// Database access layer for failed to import anime entries.
    failed_imports: FailedImports,

    /// Closure to make URL where index file can be downloaded.
    make_url: F,
}

// MARK: impl ImportIndex

impl<F> ImportIndex<F>
where 
    F: Fn(&IndexFile) -> String
{
    /// Creates new service instance.
    pub fn new(client: ImportServiceClient<Channel>, index_files: IndexFiles, failed_imports: FailedImports, make_url: F) -> Self {
        ImportIndex { client, index_files, failed_imports, make_url }
    }

    /// Starts import process.
    ///
    /// The method will wait until the import process finish and then update database
    /// with import result.
    pub async fn import(&mut self, index_file: IndexFile) -> Result<(), StateError> {
        let failed_imports = self.failed_imports.clone();
        let index_files = self.index_files.clone();
        let source = index_file.source;

        let reimport = task::spawn_blocking(move || {
            failed_imports.with_source(source)
        }).await??;

        let (new_index, old_index) = task::spawn_blocking(move || {
            let old = index_files.latest_processed(&index_file);
            (index_file, old)
        }).await?;

        let mut reimport_ids = Vec::<i32>::new();
        if let Some(ref reimport) = reimport {
            reimport_ids.extend(reimport.title_ids.iter());
        }

        let new_url = (self.make_url)(&new_index);
        let old_url = old_index?.map(|i| (self.make_url)(&i));
        let intent = ImportIntent {
            id: Some(Uuid::new()),
            source: map_source(source) as i32,
            new_index_url: new_url,
            old_index_url: old_url.unwrap_or_else(String::new),
            reimport_ids,
        };

        let res = self.client.start_import(intent).await?.into_inner();
        self.process_result(res, new_index, reimport).await
    }

    /// Updates database with import result.
    ///
    /// The method will update status of failed to import anime entries and
    /// will mark just processed index file as imported.
    async fn process_result(&self, res: ImportIntentResult, index: IndexFile, reimport: Option<FailedImport>) -> Result<(), StateError> {
        let index_files = self.index_files.clone();
        let failed_imports = self.failed_imports.clone();

        task::spawn_blocking(move || {
            if let Some(reimport) = reimport {
                let res = failed_imports.mark_reimported(reimport);
                match res {
                    Err(e) => return Err(e),
                    _ => (),
                };
            }

            if !res.skipped_ids.is_empty() {
                let res = failed_imports.create(&index, &res.skipped_ids);
                match res {
                    Err(e) => return Err(e),
                    _ => (),
                };
            }

            index_files.mark_processed(index)
        }).await??;

        Ok(())
    }
}

/// Converts domain anime source entry to protobuf's one.
fn map_source(s: entity::Source) -> data::Source {
    match s {
        entity::Source::Anidb => data::Source::Anidb,
    }
}

// MARK: impl data::Source

impl From<entity::Source> for data::Source {
    fn from(s: entity::Source) -> Self {
        match s {
            entity::Source::Anidb => data::Source::Anidb,
        }
    }
}
