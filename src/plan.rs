pub mod import;
pub mod index;
pub mod scrape;

use reqwest::{Client, Error as HttpError};
use tokio::task::JoinError;
use tonic::{transport::Error as TransportError, Status};
use tracing::{info, instrument};
use tracing_futures::Instrument;

use crate::{
    db::{
        entity::{IndexFile, Source},
        import::FailedImports,
        index::IndexFiles,
        QueryError,
    },
    proto::{
        import::import_service_client::ImportServiceClient,
        scraping::scraper_service_client::ScraperServiceClient,
    },
    settings::Service,
};

/// Errors that may happen during scraping plan execution.
#[derive(Debug)]
pub enum PlanError {
    /// Failed to access database.
    StorageError(QueryError),

    /// Failed to connect to external gRPC services.
    TransportError(TransportError),

    /// External gRPC service returned an error.
    ServiceError(Status),

    /// External HTTP service returned an error.
    HttpError(HttpError),

    /// Something unexpected happened.
    UnexpectedError(Box<dyn std::error::Error + Send>),
}

/// Represents end-to-end scraping run.
#[derive(Debug)]
pub struct ScrapePlan {
    /// Configuration for external services.
    service_config: Service,

    /// URL builder to access anime indexing service.
    url_builder: IndexURLBuilder,

    /// Database access layer to access processed or pending index files.
    index_files: IndexFiles,

    /// Database access layer to access failed to parse anime entries.
    failed_imports: FailedImports,
}

/// Builds URLs to access anime indexing service.
#[derive(Debug, Clone)]
pub struct IndexURLBuilder {
    /// Base URL of the service.
    base_url: String,

    /// Specifies for what kind of indexes build URLs for.
    source: Source,
}

// MARK: impl ScrapePlan

impl ScrapePlan {
    /// Creates new scraping plan instance.
    pub fn new(
        service_config: Service,
        url_builder: IndexURLBuilder,
        index_files: IndexFiles,
        failed_imports: FailedImports,
    ) -> Self {
        ScrapePlan {
            service_config,
            url_builder,
            index_files,
            failed_imports,
        }
    }

    /// Runs anime scraping.
    ///
    /// # Return
    ///
    /// Returns `Ok(true)` if there's more data to scrape. In that case it's fine to run
    /// the plan again. Or error in case of any errors.
    #[instrument(skip(self))]
    pub async fn run(&self) -> Result<bool, PlanError> {
        info!("trying to update index");
        let index = self.update_index().in_current_span().await?;
        if index.pending {
            info!("importing new index: {}", &index.id);
            self.import_index(index).in_current_span().await?;
        }

        info!("starting scraping data");
        self.scrape_data().in_current_span().await
    }

    /// Updates anime index by synchronizing with remote indexing service.
    ///
    /// # Return
    ///
    /// Returns latest anime index that should be used for scraping or error in case if update failed.
    /// If index's `pending` field is `true`, it should be imported by importer service first.
    async fn update_index(&self) -> Result<IndexFile, PlanError> {
        let client = Client::new();
        let check = index::UpdateIndex::new(&client, &self.index_files, &self.url_builder);
        check.latest_index().in_current_span().await
    }

    /// Asks importer service to import anime index.
    ///
    /// # Return
    ///
    /// Returns an error in case if import failed.
    async fn import_index(&self, index: IndexFile) -> Result<(), PlanError> {
        let url = self.service_config.import().url().to_string();
        let client = ImportServiceClient::connect(url).await?;
        let mut import = import::ImportIndex::new(client, &self.index_files, &self.failed_imports);
        import.start_import(index).in_current_span().await
    }

    /// Asks scraping service to start anime scraping.
    ///
    /// # Return
    ///
    /// If scraping succeeded and there's more data to scrape, `Ok(true)` is returned,
    /// `Ok(false)` is scraping succeeded and there's no more data to scrape. `Err` is
    /// returned in case if scraping failed.
    async fn scrape_data(&self) -> Result<bool, PlanError> {
        let url = self.service_config.scraper().url().to_string();
        let client = ScraperServiceClient::connect(url).await?;
        let mut scrape = scrape::ScrapeData::new(client, self.url_builder.source());
        scrape.start_scraping().in_current_span().await?;
        Ok(scrape.should_scrape())
    }
}

// MARK: impl IndexURLBuilder

impl IndexURLBuilder {
    /// Returns new builder instance.
    pub fn new(base_url: String, source: Source) -> Self {
        IndexURLBuilder { base_url, source }
    }

    /// Returns index source for which the builder creates URLs.
    pub fn source(&self) -> Source {
        self.source
    }

    /// Returns URL to get info about latest index file.
    pub fn latest(&self) -> String {
        format!("{}/{}/latest", &self.base_url, self.source_path())
    }

    /// Returns URL path component for the builder's `source` field.
    fn source_path(&self) -> &'static str {
        match self.source {
            Source::Anidb => "anidb",
        }
    }
}

// MARK: impl PlanError

impl From<Status> for PlanError {
    fn from(e: Status) -> Self {
        PlanError::ServiceError(e)
    }
}

impl From<QueryError> for PlanError {
    fn from(e: QueryError) -> Self {
        PlanError::StorageError(e)
    }
}

impl From<HttpError> for PlanError {
    fn from(e: HttpError) -> Self {
        PlanError::HttpError(e)
    }
}

impl From<JoinError> for PlanError {
    fn from(e: JoinError) -> Self {
        PlanError::UnexpectedError(Box::new(e))
    }
}

impl From<TransportError> for PlanError {
    fn from(e: TransportError) -> Self {
        PlanError::TransportError(e)
    }
}
