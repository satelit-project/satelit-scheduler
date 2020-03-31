use tokio::task;
use tonic::transport::Channel;
use tracing::{info, Span};
use tracing_futures::Instrument;

use super::PlanError;
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
}

// MARK: impl ImportIndex

impl<'a> ImportIndex<'a> {
    /// Creates new service instance.
    pub fn new(
        client: ImportServiceClient<Channel>,
        index_files: &'a IndexFiles,
        failed_imports: &'a FailedImports,
    ) -> Self {
        ImportIndex {
            client,
            index_files,
            failed_imports,
        }
    }

    /// Starts import process.
    ///
    /// The method will wait until the import process finish and then update database
    /// with import result.
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
            info!("will reimport ids: {:?}", &reimport.title_ids);
            reimport_ids.extend(reimport.title_ids.iter());
        }

        let new_url = &new_index.file_path;
        let old_url = old_index?.map(|i| i.file_path);
        let intent = ImportIntent {
            id: Some(Uuid::new()),
            source: map_source(source) as i32,
            new_index_url: new_url.to_owned(),
            old_index_url: old_url.unwrap_or_else(String::new),
            reimport_ids,
        };

        info!(
            "starting import with intent: {}",
            intent.id.as_ref().unwrap()
        );
        let res = self.client.start_import(intent).await?.into_inner();
        self.process_result(res, new_index, reimport)
            .in_current_span()
            .await
    }

    /// Updates database with import result.
    ///
    /// The method will update status of failed to import anime entries and
    /// will mark just processed index file as imported.
    async fn process_result(
        &self,
        res: ImportIntentResult,
        index: IndexFile,
        reimport: Option<FailedImport>,
    ) -> Result<(), PlanError> {
        let index_files = self.index_files.clone();
        let failed_imports = self.failed_imports.clone();

        let span = Span::current();
        task::spawn_blocking(move || {
            let _enter = span.enter();

            if let Some(reimport) = reimport {
                info!("marking reimported items: {:?}", &reimport.title_ids);
                failed_imports.mark_reimported(reimport)?;
            }

            if !res.skipped_ids.is_empty() {
                info!("memorizing failed to import items: {:?}", &res.skipped_ids);
                failed_imports.create(&index, &res.skipped_ids)?;
            }

            info!("marking index file as imported: {}", &index.id);
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
