use config::{Config, ConfigError, File};
use serde::Deserialize;
use lazy_static::lazy_static;

use std::time::Duration;

lazy_static! {
    static ref SHARED_SETTINGS: Settings = {
        Settings::new().expect("failed to read settings")
    };
}

/// Returns reference to global settings instance
pub fn shared() -> &'static Settings {
    &SHARED_SETTINGS
}

/// App settings used to configure it's state
#[derive(Debug, Deserialize)]
pub struct Settings {
    anidb: Anidb,
    db: Db,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name("config/default"))?;
        s.try_into()
    }

    pub fn anidb(&self) -> &Anidb {
        &self.anidb
    }

    pub fn db(&self) -> &Db {
        &self.db
    }
}

/// Settings for interaction with AniDB data dumps
#[derive(Debug, Deserialize)]
pub struct Anidb {
    dump_url: String,
    download_path: String,
    serve_path: String,
    reimport_path: String,
}

impl Anidb {
    /// Returns a link from where data dump can be downloaded
    pub fn dump_url(&self) -> &str {
        &self.dump_url
    }

    /// Returns path where to download data dump
    pub fn download_path(&self) -> &str {
        &self.download_path
    }

    /// Returns path from where data dump should be served via HTTP
    pub fn serve_path(&self) -> &str {
        &self.serve_path
    }

    /// Returns path to a file which tracks failed to import anime ids
    pub fn reimport_path(&self) -> &str {
        &self.reimport_path
    }
}

/// Database configuration
#[derive(Debug, Deserialize)]
pub struct Db {
    url: String,
    max_connections: u32,
    connection_timeout: u64,
}

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

/// Configuration for different gRPC services
#[derive(Debug, Deserialize)]
pub struct Service {
    import: RemoteServiceConfig,
    task: RemoteServiceConfig,
    scraper: RemoteServiceConfig,
}

impl Service {
    /// Returns configuration of `satelit-import`'s data dumps import service
    pub fn remote_import(&self) -> &RemoteServiceConfig {
        &self.import
    }

    /// Returns configuration of `satelit-import`'s scrape tasks manipulation service
    pub fn remote_task(&self) -> &RemoteServiceConfig {
        &self.task
    }

    /// Returns configuration of `satelit-scraper`'s scraping service
    pub fn remote_scraper(&self) -> &RemoteServiceConfig {
        &self.scraper
    }
}

/// Remote gRPC service configuration
#[derive(Debug, Deserialize)]
pub struct RemoteServiceConfig {
    host: String,
    port: i32,
}

impl RemoteServiceConfig {
    /// Returns service's address
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Returns service's port
    pub fn port(&self) -> i32 {
        self.port
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
