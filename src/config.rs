use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum OnMissing {
    Error,
    Warn,
    Ignore,
    Retry,
}

#[derive(Deserialize)]
pub struct PartialSettings {
    pub on_missing: Option<OnMissing>,
    pub retry_delay: Option<u64>,
    pub liveness_interval: Option<u64>,
}

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub on_missing: OnMissing,
    pub retry_delay: u64,
    pub liveness_interval: Option<u64>,
}

impl Settings {
    pub fn apply_partial(&self, partial: &Option<PartialSettings>) -> Self {
        match partial {
            None => self.clone(),
            Some(partial) => Self {
                on_missing: partial.on_missing.unwrap_or(self.on_missing),
                retry_delay: partial.retry_delay.unwrap_or(self.retry_delay),
                liveness_interval: partial.liveness_interval.or(self.liveness_interval),
            },
        }
    }
}

#[derive(Deserialize)]
pub struct Folder {
    pub inputs: Vec<String>,
    pub settings: Option<PartialSettings>,
}

#[derive(Deserialize)]
pub struct Config {
    pub settings: Settings,
    pub folders: HashMap<String, Folder>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let path = std::fs::canonicalize(path)
            .with_context(|| format!("Couldn't get true path of {path:?}"))?;
        let file = File::open(&path).with_context(|| format!("Couldn't open {path:?}"))?;
        let reader = BufReader::new(file);
        serde_yaml::from_reader(reader).with_context(|| format!("Failed to parse {path:?}"))
    }
}
