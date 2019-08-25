use futures::prelude::*;
use reqwest::r#async::Client;
use serde::{Deserialize, Serialize};

use super::StateError;
use crate::block::blocking;
use crate::db::entity::IndexFile;
use crate::db::index::IndexFiles;

pub struct CheckIndex<'a> {
    client: Client,
    store: IndexFiles,
    index_url: &'a str
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NewIndexFile {
    url: String,
    hash: String,
}

impl<'a> CheckIndex<'a> {
    pub fn new(client: Client, store: IndexFiles, index_url: &'a str) -> Self {
        CheckIndex { client, store, index_url }
    }

    pub fn latest_index(&self) -> impl Future<Item = IndexFile, Error = StateError> + Send {
        let store = self.store.clone();
        self.request_index()
            .from_err()
            .and_then(move |new_index| blocking(move || store.queue(&new_index.hash)).from_err())
    }

    fn request_index(&self) -> impl Future<Item = NewIndexFile, Error = reqwest::Error> + Send {
        self.client
            .get(self.index_url)
            .send()
            .and_then(|r| r.error_for_status())
            .and_then(|mut r| r.json::<NewIndexFile>())
    }
}
