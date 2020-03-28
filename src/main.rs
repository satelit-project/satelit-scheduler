use std::time::Duration;

use tokio::time;
use tracing::{error, info, info_span};
use tracing_futures::Instrument as _;
use tracing_subscriber::{filter::LevelFilter, FmtSubscriber};

use satelit_scheduler::{
    db::{self, entity::Source, import::FailedImports, index::IndexFiles},
    plan::{IndexURLBuilder, ScrapePlan},
    settings::Settings,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(LevelFilter::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("loading configuration");
    let config = Settings::new()?;
    let url_builder =
        IndexURLBuilder::new(config.services().indexer().url().to_string(), Source::Anidb);

    info!("connecting to database");
    let pool = db::new_connection_pool(config.db())?;
    let index_files = IndexFiles::new(pool.clone());
    let failed_imports = FailedImports::new(pool);

    loop {
        let services = config.services().clone();
        let url_builder = url_builder.clone();
        let index_files = index_files.clone();
        let failed_imports = failed_imports.clone();

        let scrape_runner = async move {
            info!("running scraping plan");
            let plan = ScrapePlan::new(services, url_builder, index_files, failed_imports);
            let res = plan.run().await;

            match res {
                Ok(more) => {
                    info!("scrape succeeded, has more data to srape: {}", more);
                    if !more {
                        info!("nothing to scrape anymore, waiting for 24h");
                        time::delay_for(Duration::from_secs(24 * 60 * 60)).await;
                    }
                }
                Err(e) => {
                    error!("scraping plan failed: {:?}", e);
                    time::delay_for(Duration::from_secs(60)).await;
                }
            }
        };

        scrape_runner.instrument(info_span!("plan")).await;
    }
}
