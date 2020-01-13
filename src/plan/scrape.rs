use tonic::transport::Channel;
use tracing::{debug, instrument};

use super::PlanError;
use crate::{
    db::entity::Source,
    proto::{
        scraping::{scraper_service_client::ScraperServiceClient, ScrapeIntent},
        uuid::Uuid,
    },
};

/// Asks scraping RPC service to start anime scraping.
pub struct ScrapeData {
    /// RPC service client.
    client: ScraperServiceClient<Channel>,

    /// From where to scrape data.
    source: Source,

    /// Indicates if there's data for scraping service to scrape.
    should_scrape: bool,
}

// MARK: impl ScrapeData

impl ScrapeData {
    /// Creates new struct instance.
    pub fn new(client: ScraperServiceClient<Channel>, source: Source) -> Self {
        ScrapeData {
            client,
            source,
            should_scrape: true,
        }
    }

    /// Returns `true` if there's data to scrape of `false` otherwise.
    ///
    /// If `false` is returned that means there's no scraping will be performed
    /// on subsequent calls to `start_scraping`, presumably. RPC call will be made
    /// anyway and it's possible that returned value is out of date. So you should
    /// use it only as a suggestion.
    pub fn should_scrape(&self) -> bool {
        self.should_scrape
    }

    /// Start scraping process and waits until it's done.
    ///
    /// If it's not all data has been scraped, `should_scrape()` will
    /// return `true`. In that case feel free to call this method again.
    /// It's still safe to call the method again if `should_scrape()`
    /// returns `false`. The RPC call will be made but scraper service
    /// may return immediatelly.
    #[instrument(skip(self))]
    pub async fn start_scraping(&mut self) -> Result<(), PlanError> {
        let intent = ScrapeIntent {
            id: Some(Uuid::new()),
            source: self.source as i32,
        };

        debug!("starting scraping with intent: {:?}", &intent);
        let res = self.client.start_scraping(intent).await?;
        self.should_scrape = res.get_ref().may_continue;

        Ok(())
    }
}
