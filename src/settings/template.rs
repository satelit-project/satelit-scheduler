use serde::{Deserialize, Serialize};
use tinytemplate as tt;

use std::{env, fs, io::Read, path::Path};

/// Represents a template configuration file.
#[derive(Debug)]
pub struct TemplateConfig<P> {
    /// Path to the configuration file.
    path: P,

    /// Variables to substitute.
    env: Env,
}

/// Represents data to be substituted in a configuration file template.
#[derive(Debug, Serialize, Deserialize)]
pub struct Env {
    /// Database configuration.
    db: Option<Db>,
}

/// Represents database configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct Db {
    /// Database URL.
    url: String,
}

// MARK: impl ConfigFile

impl<P> TemplateConfig<P>
where
    P: AsRef<Path> + 'static,
{
    /// Creates new configuration file.
    pub fn new(path: P) -> Self {
        Self::with_env(path, Env::default())
    }

    /// Creates new configuration file with custom environment.
    pub fn with_env(path: P, env: Env) -> Self {
        TemplateConfig { path, env }
    }

    /// Reads and renders configuration with environment data.
    pub fn render(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut tf = fs::File::open(&self.path)?;
        let mut raw = String::new();
        tf.read_to_string(&mut raw)?;

        let mut tmpl = tt::TinyTemplate::new();
        tmpl.add_template("cfg", &raw)?;

        Ok(tmpl.render("cfg", &self.env)?)
    }
}

// MARK: impl Env

impl Default for Env {
    fn default() -> Self {
        Env {
            db: Db::from_env(),
        }
    }
}

// MARK: impl Db

impl Db {
    fn from_env() -> Option<Self> {
        let url = env::var("PG_DB_URL").ok()?;
        Some(Db { url })
    }
}
