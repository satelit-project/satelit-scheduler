use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::task;
use tracing::{debug, info};

use std::convert::{TryFrom, TryInto};

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

    /// Path to the file in an index storage.
    file_path: String,

    /// Type of DB index file relates to.
    source: i32,
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
    pub async fn latest_index(&self) -> Result<IndexFile, PlanError> {
        let url = self.url_builder.latest();
        info!("requesting latest index from {}", &url);

        let resp = self.client.get(&url).send().await?.error_for_status()?;
        let new_index = resp.json::<NewIndexFile>().await?;
        let source = new_index.source.try_into()?;
        info!("received new index: {}", &new_index.id);

        let store = self.store.clone();
        let index = task::spawn_blocking(move || {
            let path = &new_index.file_path;
            let source = source;
            store.queue(path, source)
        })
        .await??;

        debug!(pending = index.pending);
        Ok(index)
    }
}

// MARK: impl i32

impl TryFrom<i32> for Source {
    type Error = PlanError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Source::Anidb),
            _ => Err(PlanError::ServiceError(super::Status::internal(
                "failed to convert index source",
            ))),
        }
    }
}
