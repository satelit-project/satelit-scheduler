use std::time::Duration;

use tokio::time;

use satelit_scheduler::{
    db::{self, entity::Source, import::FailedImports, index::IndexFiles},
    plan::{IndexURLBuilder, ScrapePlan},
    settings::Settings,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Settings::new()?;
    let url_builder = IndexURLBuilder::new(
        config.services().indexer().url().to_string(),
        config.index_url().clone(),
        Source::Anidb,
    );

    let pool = db::new_connection_pool(config.db())?;
    let index_files = IndexFiles::new(pool.clone());
    let failed_imports = FailedImports::new(pool);

    let plan = ScrapePlan::new(
        config.services().clone(),
        url_builder,
        index_files,
        failed_imports,
    );

    loop {
        let res = plan.run().await;
        match res {
            Ok(more) => {
                if !more {
                    time::delay_for(Duration::from_secs(24 * 60 * 60)).await;
                }
            }
            Err(_e) => {
                // TODO: log error
                time::delay_for(Duration::from_secs(60 * 60)).await;
            }
        }
    }
}
