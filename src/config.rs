use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Folder {
    pub inputs: Vec<String>,
}

#[derive(Deserialize)]
pub struct Config {
    pub folders: HashMap<String, Folder>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let path = std::fs::canonicalize(path)?;
        let file =
            File::open(&path).with_context(|| format!("Couldn't open {}", path.display()))?;
        let reader = BufReader::new(file);
        serde_yaml::from_reader(reader)
            .with_context(|| format!("Failed to parse {}", path.display()))
    }
}
