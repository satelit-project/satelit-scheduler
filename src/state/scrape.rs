use tonic::transport::Channel;

use super::StateError;
use crate::db::entity::Source;
use crate::proto::uuid::Uuid;
use crate::proto::scraping::ScrapeIntent;
use crate::proto::scraping::scraper_service_client::ScraperServiceClient;

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
        ScrapeData { client, source, should_scrape: true }
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
    pub async fn start_scraping(&mut self) -> Result<(), StateError> {
        let intent = ScrapeIntent {
            id: Some(Uuid::new()),
            source: self.source as i32,
        };

        let res = self.client.start_scraping(intent).await?;
        self.should_scrape = res.get_ref().may_continue;

        Ok(())
    }
}
