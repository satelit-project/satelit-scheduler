use serde::Deserialize;
use config::{Config, ConfigError, File};

use std::sync::Once;

/// Returns reference to global settings instance
pub fn shared() -> &'static Settings {
    static mut SHARED: *const Settings = std::ptr::null();
    static ONCE: Once = Once::new();

    unsafe {
        ONCE.call_once(|| {
            let settings = Settings::new().expect("failed to read settings");
            SHARED = Box::into_raw(Box::new(settings));
        });

        &*SHARED
    }
}

/// App settings used to configure it's state
#[derive(Debug, Deserialize)]
pub struct Settings {
    anidb: Anidb,
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
