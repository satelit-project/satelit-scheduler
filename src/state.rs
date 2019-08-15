pub mod import;
pub mod index;
pub mod scrape;

use futures::prelude::*;
use futures::task;
use futures::try_ready;
use reqwest::{r#async::Client, Error as HttpError};
use tower_grpc::client;
use tower_grpc::generic::client::GrpcService;
use tower_grpc::Status;

use std::marker::PhantomData;

use crate::block::BlockingError;
use crate::db::entity::IndexFile;
use crate::db::import::FailedImports;
use crate::db::index::IndexFiles;
use crate::db::QueryError;
use crate::proto::import::client::ImportService;
use crate::proto::import::ImportIntent;
use crate::proto::scraping::client::ScraperService;
use crate::proto::scraping::ScrapeIntent;
use crate::state::scrape::ScrapeData;

#[derive(Debug)]
pub enum StateError {
    StorageError(QueryError),
    GrpcError(Status),
    HttpError(HttpError),
    Timeout,
    UnknownError(Box<dyn std::error::Error>),
}

enum ScrapeState<T, R> {
    Initial,
    CheckIndex(Box<dyn Future<Item = IndexFile, Error = StateError>>),
    ImportIndex(Box<dyn Future<Item = import::ImportIndex<T, R>, Error = StateError>>),
    ScrapeData(Box<dyn Future<Item = scrape::ScrapeData<T, R>, Error = StateError>>),
}

pub struct ScrapePlan<T, R> {
    state: ScrapeState<T, R>,
    index_files: IndexFiles,
    failed_imports: FailedImports,
    import_service: Option<ImportService<T>>,
    scraper_service: Option<ScraperService<T>>,
    http_client: Option<Client>,
    _request: PhantomData<R>,
}

impl<T, R> ScrapePlan<T, R>
where
    T: GrpcService<R>,
    client::unary::Once<ImportIntent>: client::Encodable<R>,
    client::unary::Once<ScrapeIntent>: client::Encodable<R>,
{
    pub fn new(
        index_files: IndexFiles,
        failed_imports: FailedImports,
        import_service: ImportService<T>,
        scraper_service: ScraperService<T>,
        http_client: Client,
    ) -> Self {
        ScrapePlan {
            state: ScrapeState::Initial,
            index_files,
            failed_imports,
            import_service: Some(import_service),
            scraper_service: Some(scraper_service),
            http_client: Some(http_client),
            _request: PhantomData,
        }
    }

    fn check_index(&mut self) -> impl Future<Item = IndexFile, Error = StateError> {
        let client = self
            .http_client
            .take()
            .expect("index should be checked once");

        let check_index = index::CheckIndex::new(client, self.index_files.clone());
        check_index.latest_index()
    }

    fn import_index(
        &mut self,
        index: IndexFile,
    ) -> impl Future<Item = import::ImportIndex<T, R>, Error = StateError> {
        let import_service = self
            .import_service
            .take()
            .expect("import should happen only once");

        let import_index = import::ImportIndex::new(
            import_service,
            self.index_files.clone(),
            self.failed_imports.clone(),
        );

        import_index.import(index)
    }

    fn scrape_data(&mut self) -> impl Future<Item = scrape::ScrapeData<T, R>, Error = StateError> {
        let scraper_service = self
            .scraper_service
            .take()
            .expect("scraping should happen only once");

        let scrape_data = scrape::ScrapeData::new(scraper_service);
        scrape_data.start_scraping()
    }

    fn repeat_scrape_data(
        &mut self,
        scrape_data: ScrapeData<T, R>,
    ) -> impl Future<Item = scrape::ScrapeData<T, R>, Error = StateError> {
        scrape_data.start_scraping()
    }
}

impl<T, R> Future for ScrapePlan<T, R>
where
    T: GrpcService<R> + 'static,
    R: 'static,
    client::unary::Once<ImportIntent>: client::Encodable<R>,
    client::unary::Once<ScrapeIntent>: client::Encodable<R>,
{
    type Item = ();
    type Error = StateError;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        match &mut self.state {
            ScrapeState::Initial => {
                let fut = self.check_index();
                self.state = ScrapeState::CheckIndex(Box::new(fut));
                task::current().notify();
                Ok(Async::NotReady)
            }
            ScrapeState::CheckIndex(check_index) => {
                let index_file = try_ready!(check_index.poll());
                if index_file.pending {
                    let fut = self.import_index(index_file);
                    self.state = ScrapeState::ImportIndex(Box::new(fut));
                } else {
                    let fut = self.scrape_data();
                    self.state = ScrapeState::ScrapeData(Box::new(fut));
                }

                task::current().notify();
                Ok(Async::NotReady)
            }
            ScrapeState::ImportIndex(import_index) => {
                let _ = try_ready!(import_index.poll());
                let fut = self.scrape_data();
                self.state = ScrapeState::ScrapeData(Box::new(fut));
                task::current().notify();
                Ok(Async::NotReady)
            }
            ScrapeState::ScrapeData(scrape_data) => {
                let state = try_ready!(scrape_data.poll());
                if !state.should_scrape() {
                    return Ok(Async::Ready(()));
                }

                let fut = self.repeat_scrape_data(state);
                self.state = ScrapeState::ScrapeData(Box::new(fut));
                task::current().notify();
                Ok(Async::NotReady)
            }
        }
    }
}

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
