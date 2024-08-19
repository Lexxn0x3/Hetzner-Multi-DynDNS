use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug)]
pub enum ConfigError {
    IoError(()),
    ParseError(()),
}

impl From<io::Error> for ConfigError {
    fn from(_: io::Error) -> Self {
        ConfigError::IoError(())
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(_: toml::de::Error) -> Self {
        ConfigError::ParseError(())
    }
}

#[derive(Deserialize)]
pub struct Config {
    pub api_token: String,
    pub interval_secs: u64,
    pub log_level: String,
    pub records: Vec<RecordConfig>,
}

#[derive(Deserialize)]
pub struct RecordConfig {
    pub record_id: String,
    pub name: String,
    pub ttl: Option<u64>,
    pub record_type: String,
    pub zone_id: String,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let config_content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&config_content)?;
        Ok(config)
    }
}

