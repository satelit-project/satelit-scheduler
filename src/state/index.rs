use futures::prelude::*;
use serde::{Serialize, Deserialize};
use reqwest::r#async::Client as HttpClient;

use std::error::Error;

use crate::block::{blocking, BlockingError};
use crate::settings;

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct IndexFile {
    url: String,
    hash: String,
}

pub trait IndexClient {
    type Error: std::error::Error + Send;
    type Future: Future<Item = IndexFile, Error = Self::Error> + Send;

    fn latest_index(&self) -> Self::Future;
}

pub trait IndexStore: Clone + Send {
    type Error: std::error::Error + Send;

    fn is_outdated(&self, index: &IndexFile) -> Result<bool, Self::Error>;
    fn set_latest(&mut self, index: &IndexFile) -> Result<(), Self::Error>;
}

pub struct CheckIndex<C, S> {
    client: C,
    store: S,
}

impl<C, S> CheckIndex<C, S>
where
    C: IndexClient + 'static,
    S: IndexStore + 'static,
{
    pub fn new(client: C, store: S) -> Self {
        CheckIndex {
            client,
            store,
        }
    }

    pub fn updated_index(&self) -> impl Future<Item = Option<IndexFile>, Error = IndexError> {
        let store = self.store.clone();
        self.client.latest_index()
            .map_err(|e| IndexError::NetworkError(Box::new(e)))
            .and_then(move |index| {
                let index_cloned = index.clone();
                let outdated = blocking( move || store.is_outdated(&index))
                    .map_err(|e: BlockingError<S::Error>| IndexError::StorageError(Box::new(e)));

                outdated.join(Ok(index_cloned))
            })
            .and_then(|(outdated, index)| {
                if outdated { Ok(None) } else { Ok(Some(index)) }
            })
    }
}

pub enum IndexError {
    NetworkError(Box<dyn Error>),
    StorageError(Box<dyn Error>),
}

impl IndexClient for HttpClient {
    type Error = reqwest::Error;
    type Future = Box<dyn Future<Item = IndexFile, Error = Self::Error> + Send>;

    fn latest_index(&self) -> Self::Future {
        let url = settings::shared().anidb().dump_url();
        let fut = self.get(url).send()
            .and_then(|r| r.error_for_status().into_future())
            .and_then(|mut r| r.json::<IndexFile>());

        Box::new(fut)
    }
}
