use futures::prelude::*;
use reqwest::r#async::Client;
use serde::{Deserialize, Serialize};

use super::StateError;
use crate::block::blocking;
use crate::db::entity::IndexFile;
use crate::db::index::IndexFiles;
use crate::settings;

pub struct CheckIndex {
    client: Client,
    store: IndexFiles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NewIndexFile {
    url: String,
    hash: String,
}

impl CheckIndex {
    pub fn new(client: Client, store: IndexFiles) -> Self {
        CheckIndex { client, store }
    }

    pub fn latest_index(&self) -> impl Future<Item = IndexFile, Error = StateError> {
        let store = self.store.clone();
        self.request_index()
            .from_err()
            .and_then(move |new_index| blocking(move || store.queue(&new_index.hash)).from_err())
    }

    fn request_index(&self) -> impl Future<Item = NewIndexFile, Error = reqwest::Error> {
        let url = settings::shared().anidb().dump_url();
        self.client
            .get(url)
            .send()
            .and_then(|r| r.error_for_status())
            .and_then(|mut r| r.json::<NewIndexFile>())
    }
}
