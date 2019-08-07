use futures::prelude::*;
use reqwest::r#async::Client;
use serde::{Deserialize, Serialize};

use std::error::Error;

use crate::settings;
use crate::block::blocking;
use crate::db::index::IndexFiles;
use crate::db::entity::IndexFile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewIndexFile {
    url: String,
    hash: String,
}

pub enum CheckError {
    NetworkError(Box<dyn Error>),
    StorageError(Box<dyn Error>),
}

pub struct CheckIndex {
    client: Client,
    store: IndexFiles,
}

impl CheckIndex {
    pub fn new(client: Client, store: IndexFiles) -> Self {
        CheckIndex { client, store }
    }

    pub fn updated_index(&self) -> impl Future<Item = IndexFile, Error = CheckError> {
        let store = self.store.clone();
        self.latest_index()
            .map_err(|e| CheckError::NetworkError(Box::new(e)))
            .and_then(move |new_index| {
                blocking(move || {
                    store.queue(&new_index.hash)
                }).map_err(|e| CheckError::StorageError(Box::new(e)))
            })
    }

    fn latest_index(&self) -> impl Future<Item = NewIndexFile, Error = reqwest::Error> {
        let url = settings::shared().anidb().dump_url();
        self.client
            .get(url)
            .send()
            .and_then(|r| r.error_for_status())
            .and_then(|mut r| r.json::<NewIndexFile>())
    }
}
