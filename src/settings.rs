use config::{Config, ConfigError, File};
use lazy_static::lazy_static;
use serde::Deserialize;

use std::time::Duration;

lazy_static! {
    static ref SHARED_SETTINGS: Settings = { Settings::new().expect("failed to read settings") };
}

/// Returns reference to global settings instance
pub fn shared() -> &'static Settings {
    &SHARED_SETTINGS
}

/// App settings used to configure it's state
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    services: Service,
    db: Db,
    index_url: IndexURL,
}

/// Database configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Db {
    url: String,
    max_connections: u32,
    connection_timeout: u64,
}

/// Configuration for different gRPC services
#[derive(Debug, Clone, Deserialize)]
pub struct Service {
    indexer: RemoteServiceConfig,
    import: RemoteServiceConfig,
    scraper: RemoteServiceConfig,
}

/// Remote gRPC service configuration
#[derive(Debug, Clone, Deserialize)]
pub struct RemoteServiceConfig {
    url: String,
    connection_timeout: Option<i32>,
    request_timeout: Option<i32>,
}

/// URL templates for index files requests.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename(deserialize = "index_url"))]
pub struct IndexURL {
    latest: String,
    index_file: String,
}

// MARK: impl Settings

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name("config/default"))?;
        s.try_into()
    }

    pub fn services(&self) -> &Service {
        &self.services
    }

    pub fn db(&self) -> &Db {
        &self.db
    }

    pub fn index_url(&self) -> &IndexURL {
        &self.index_url
    }
}

// MARK: impl Db

impl Db {
    /// Returns database connection URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Return number of maximum database connections
    pub fn max_connections(&self) -> u32 {
        self.max_connections
    }

    /// Returns database connection timeout
    pub fn connection_timeout(&self) -> Duration {
        Duration::new(self.connection_timeout, 0)
    }
}

// MARK: impl Service

impl Service {
    /// Returns configuration for `satelit-index` indexer service
    pub fn indexer(&self) -> &RemoteServiceConfig {
        &self.indexer
    }

    /// Returns configuration for `satelit-import` index import service
    pub fn import(&self) -> &RemoteServiceConfig {
        &self.import
    }

    /// Returns configuration for `satelit-scraper` scraping service
    pub fn scraper(&self) -> &RemoteServiceConfig {
        &self.scraper
    }
}

// MARK: impl RemoteServiceConfig

impl RemoteServiceConfig {
    /// Returns service's URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns preferred connection timeout
    pub fn connection_timeout(&self) -> Option<i32> {
        self.connection_timeout
    }

    /// Returns preferred request timeout
    pub fn request_timeout(&self) -> Option<i32> {
        self.request_timeout
    }
}

// MARK: impl IndexURL

impl IndexURL {
    /// Returns template for requesting latest index files info.
    pub fn latest(&self) -> &str {
        &self.latest
    }

    /// Returns template for downloading specific index file.
    pub fn index_file(&self) -> &str {
        &self.index_file
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parsing() {
        // if this does not panic then everything is good
        let _ = super::shared();
    }
}
