#[macro_use]
extern crate diesel;

pub mod db;
pub mod proto;
pub mod settings;
pub mod plan;

use futures::prelude::*;
// use tokio::prelude::*;
// use tower::builder::ServiceBuilder;
// use tower::MakeService;
// use tower_hyper::util::{Destination, HttpConnector};
// use tower_hyper::{client, util};

use std::time::Duration;

use db::{import::FailedImports, index::IndexFiles};
// use proto::import::client::ImportService;
// use proto::scraping::client::ScraperService;

// TODO: do I need `tower_request_modifier`?
pub fn try_use() {
    //     let uri1: hyper::Uri = "http://localhost:10000".parse().unwrap();
    //     let dst1 = Destination::try_from_uri(uri1).unwrap();
    //
    //     let uri2: hyper::Uri = "http://localhost:10000".parse().unwrap();
    //     let dst2 = Destination::try_from_uri(uri2).unwrap();
    //
    //     let http_client = reqwest::r#async::ClientBuilder::new()
    //         .timeout(Duration::new(60, 0))
    //         .build()
    //         .unwrap();
    //
    //     let mut connector = HttpConnector::new(4);
    //     connector.set_keepalive(Some(Duration::new(60, 0)));
    //
    //     let settings = client::Builder::new().http2_only(true).clone();
    //     let mut make_client = client::Connect::with_builder(util::Connector::new(connector), settings);
    //
    //     let import_service = make_client
    //         .make_service(dst1)
    //         .map_err(|e| panic!("{:?}", e))
    //         .and_then(|conn| {
    //             let timeout_conn = ServiceBuilder::new()
    //                 .timeout(Duration::new(1 * 60 * 60, 0))
    //                 .service(conn);
    //
    //             Ok(ImportService::new(timeout_conn))
    //         });
    //
    //     let scraper_service = make_client
    //         .make_service(dst2)
    //         .map_err(|e| panic!("{:?}", e))
    //         .and_then(|conn| {
    //             let timeout_conn = ServiceBuilder::new()
    //                 .timeout(Duration::new(1 * 60 * 60, 0))
    //                 .service(conn);
    //
    //             Ok(ScraperService::new(timeout_conn))
    //         });
    //
    //     let run =
    //         import_service
    //             .join(scraper_service)
    //             .and_then(move |(import_service, scraper_service)| {
    //                 let pool = db::connection_pool();
    //                 let index_files = IndexFiles::new(pool.clone());
    //                 let failed_imports = FailedImports::new(pool.clone());
    //
    //                 let scrape = state::ScrapePlan::new(
    //                     settings::shared().services().indexer().url(),
    //                     index_files,
    //                     failed_imports,
    //                     import_service,
    //                     scraper_service,
    //                     http_client,
    //                 );
    //
    //                 scrape.timeout(Duration::new(24 * 60 * 60, 0))
    //             });
    //
    //     tokio::run(run.map_err(|e| panic!("{:?}", e)));
}
