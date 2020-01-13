use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::task;
use tracing::{debug, instrument};

use super::{IndexURLBuilder, PlanError};
use crate::db::{
    entity::{IndexFile, Source},
    index::IndexFiles,
};

/// Service that fetches latest anime index files.
pub struct UpdateIndex<'a> {
    /// HTTP client.
    client: &'a Client,

    /// DB access layer for anime index files.
    store: &'a IndexFiles,

    /// URL to get info about latest index files.
    url_builder: &'a IndexURLBuilder,
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
    pub fn new(
        client: &'a Client,
        store: &'a IndexFiles,
        url_builder: &'a IndexURLBuilder,
    ) -> Self {
        UpdateIndex {
            client,
            store,
            url_builder,
        }
    }

    /// Updates and returns latest anime index file.
    ///
    /// In case if there's new index file available it will be saved to DB with
    /// `pending == true` status. Otherwise, existing record from the DB will be returned.
    #[instrument(skip(self))]
    pub async fn latest_index(&self) -> Result<IndexFile, PlanError> {
        let url = self.url_builder.latest()?;
        debug!("requesting latest index from {}", &url);

        let resp = self.client.get(&url).send().await?.error_for_status()?;
        let new_index = resp.json::<NewIndexFile>().await?;
        debug!("received new index: {}", &new_index.id);

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
