use futures::prelude::*;
use tower_grpc::client::{unary, Encodable};
use tower_grpc::generic::client::GrpcService;
use tower_grpc::Request;
use uuid::Uuid;

use std::marker::PhantomData;

use super::StateError;
use crate::db::entity;
use crate::proto::scraping::client::ScraperService;
use crate::proto::scraping::ScrapeIntent;

pub struct ScrapeData<T, R> {
    client: ScraperService<T>,
    _request: PhantomData<R>,
}

impl<T, R> ScrapeData<T, R>
where
    T: GrpcService<R>,
    unary::Once<ScrapeIntent>: Encodable<R>,
{
    pub fn new(client: ScraperService<T>) -> Self {
        Self {
            client,
            _request: PhantomData,
        }
    }

    pub fn start_scraping(self) -> impl Future<Item = Self, Error = StateError> {
        let ScrapeData { client, .. } = self;
        client.ready().from_err().and_then(|mut client| {
            let intent = ScrapeIntent {
                id: Uuid::new_v4().to_string(),
                source: entity::Source::Anidb as i32,
            };
            let request = Request::new(intent);

            client
                .start_scraping(request)
                .from_err()
                .and_then(move |_| {
                    let me = Self::new(client);
                    Ok(me)
                })
        })
    }
}
