use serde::{Serialize, Deserialize};
use futures::prelude::*;

use std::error::Error;

use crate::block::{blocking, BlockingError};

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
