use tokio::task;
use tonic::transport::Channel;
use tracing::{debug, instrument};

use super::{IndexURLBuilder, PlanError};
use crate::{
    db::{
        entity::{self, FailedImport, IndexFile},
        import::FailedImports,
        index::IndexFiles,
    },
    proto::{
        data,
        import::{import_service_client::ImportServiceClient, ImportIntent, ImportIntentResult},
        uuid::Uuid,
    },
};

/// Ask import service to start importing new database index file.
pub struct ImportIndex<'a> {
    /// RPC client for importing service.
    client: ImportServiceClient<Channel>,

    /// Database access layer for all index files.
    index_files: &'a IndexFiles,

    /// Database access layer for failed to import anime entries.
    failed_imports: &'a FailedImports,

    /// URL builder for index file downloads.
    url_builder: &'a IndexURLBuilder,
}

// MARK: impl ImportIndex

impl<'a> ImportIndex<'a> {
    /// Creates new service instance.
    pub fn new(
        client: ImportServiceClient<Channel>,
        index_files: &'a IndexFiles,
        failed_imports: &'a FailedImports,
        url_builder: &'a IndexURLBuilder,
    ) -> Self {
        ImportIndex {
            client,
            index_files,
            failed_imports,
            url_builder,
        }
    }

    /// Starts import process.
    ///
    /// The method will wait until the import process finish and then update database
    /// with import result.
    #[instrument(skip(self))]
    pub async fn start_import(&mut self, index_file: IndexFile) -> Result<(), PlanError> {
        let failed_imports = self.failed_imports.clone();
        let index_files = self.index_files.clone();
        let source = index_file.source;
        let reimport = task::spawn_blocking(move || failed_imports.with_source(source)).await??;

        let (new_index, old_index) = task::spawn_blocking(move || {
            let old = index_files.latest_processed(&index_file);
            (index_file, old)
        })
        .await?;

        let mut reimport_ids = Vec::<i32>::new();
        if let Some(ref reimport) = reimport {
            reimport_ids.extend(reimport.title_ids.iter());
        }

        let new_url = self.url_builder.index(&new_index);
        let old_url = old_index?.map(|i| self.url_builder.index(&i));
        let intent = ImportIntent {
            id: Some(Uuid::new()),
            source: map_source(source) as i32,
            new_index_url: new_url,
            old_index_url: old_url.unwrap_or_else(String::new),
            reimport_ids,
        };

        debug!("starting import with indent: {:?}", &intent);
        let res = self.client.start_import(intent).await?.into_inner();
        self.process_result(res, new_index, reimport).await
    }

    /// Updates database with import result.
    ///
    /// The method will update status of failed to import anime entries and
    /// will mark just processed index file as imported.
    #[instrument(skip(self))]
    async fn process_result(
        &self,
        res: ImportIntentResult,
        index: IndexFile,
        reimport: Option<FailedImport>,
    ) -> Result<(), PlanError> {
        let index_files = self.index_files.clone();
        let failed_imports = self.failed_imports.clone();

        task::spawn_blocking(move || {
            if let Some(reimport) = reimport {
                debug!("marking reimported items");
                if let Err(e) = failed_imports.mark_reimported(reimport) {
                    return Err(e);
                }
            }

            if !res.skipped_ids.is_empty() {
                debug!("memorizing failed to import items");
                if let Err(e) = failed_imports.create(&index, &res.skipped_ids) {
                    return Err(e);
                }
            }

            debug!("marking index file as imported");
            index_files.mark_processed(index)
        })
        .await??;

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
