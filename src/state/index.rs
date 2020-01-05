use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::task;

use super::StateError;
use crate::db::entity::{IndexFile, Source};
use crate::db::index::IndexFiles;

/// Service that fetches latest anime index files.
pub struct UpdateIndex<'a> {
    /// HTTP client.
    client: Client,

    /// DB access layer for anime index files.
    store: IndexFiles,

    /// URL to get info about latest index files.
    index_url: &'a str,
}

/// Represents lates anime index file.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NewIndexFile {
    /// Index file identifier.
    id: String,

    /// Hash of the index file.
    hash: String,

    /// Type of DB index file relates to.
    source: NewIndexFileSource,
}

/// Represents type of DB an index file relates to.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum NewIndexFileSource {
    Anidb,
}

// MARK: impl UpdateIndex

impl<'a> UpdateIndex<'a> {
    pub fn new(client: Client, store: IndexFiles, index_url: &'a str) -> Self {
        UpdateIndex {
            client,
            store,
            index_url,
        }
    }

    /// Updates and returns latest anime index file.
    ///
    /// In case if there's new index file available it will be saved to DB with
    /// `pending == true` status. Otherwise, existing record from the DB will be returned.
    pub async fn latest_index(&self) -> Result<IndexFile, StateError> {
        let resp = self
            .client
            .get(self.index_url)
            .send()
            .await?
            .error_for_status()?;
        let new_index = resp.json::<NewIndexFile>().await?;

        let store = self.store.clone();
        let index = task::spawn_blocking(move || {
            let hash = &new_index.hash;
            let source = new_index.source.into();
            store.queue(hash, source)
        })
        .await??;
        Ok(index)
    }
}

// MARK: impl NewIndexFileSource

impl From<NewIndexFileSource> for Source {
    fn from(s: NewIndexFileSource) -> Self {
        match s {
            NewIndexFileSource::Anidb => Source::Anidb,
        }
    }
}
