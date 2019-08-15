#[macro_use]
extern crate diesel;

pub mod block;
pub mod db;
pub mod proto;
pub mod settings;
pub mod state;

use futures::prelude::*;
use tower::MakeService;
use tower_hyper::util::{Destination, HttpConnector};
use tower_hyper::{client, util};

use db::{import::FailedImports, index::IndexFiles};
use proto::import::client::ImportService;
use proto::scraping::client::ScraperService;

pub fn try_use() {
    let uri1: hyper::Uri = "http://localhost:10000".parse().unwrap();
    let dst1 = Destination::try_from_uri(uri1).unwrap();

    let uri2: hyper::Uri = "http://localhost:10000".parse().unwrap();
    let dst2 = Destination::try_from_uri(uri2).unwrap();

    let connector = util::Connector::new(HttpConnector::new(4));
    let settings = client::Builder::new().http2_only(true).clone();

    let http_client = reqwest::r#async::ClientBuilder::new().build().unwrap();
    let mut make_client = client::Connect::with_builder(connector, settings);

    let import_service = make_client
        .make_service(dst1)
        .map_err(|e| panic!("{:?}", e))
        .and_then(|conn| Ok(ImportService::new(conn)));

    let scraper_service = make_client
        .make_service(dst2)
        .map_err(|e| panic!("{:?}", e))
        .and_then(|conn| Ok(ScraperService::new(conn)));

    let run =
        import_service
            .join(scraper_service)
            .and_then(move |(import_service, scraper_service)| {
                let pool = db::connection_pool();
                let index_files = IndexFiles::new(pool.clone());
                let failed_imports = FailedImports::new(pool.clone());

                state::ScrapePlan::new(
                    index_files,
                    failed_imports,
                    import_service,
                    scraper_service,
                    http_client,
                )
            });

    tokio::run(run.map_err(|e| panic!("{:?}", e)));
}
