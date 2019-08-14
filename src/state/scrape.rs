use futures::prelude::*;
use tower_grpc::client::{unary, Encodable};
use tower_grpc::generic::client::GrpcService;

use std::marker::PhantomData;

use crate::proto::scraping::client::ScraperService;
use crate::proto::scraping::ScrapeIntent;

pub struct Scrape<T, R> {
    client: ScraperService<T>,
    _request: PhantomData<R>,
}

impl<T, R> Scrape<T, R>
where
    T: GrpcService<R>,
    unary::Once<ScrapeIntent>: Encodable<R>,
{
}
