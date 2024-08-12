use std::{fs::canonicalize, path::{Path, PathBuf}, str::from_utf8};

use tilde_expand::tilde_expand;
use anyhow::{Context, Result};



pub fn get_path(path: &str) -> Result<PathBuf> {
  let expanded_s = tilde_expand(path.as_bytes());
  let expanded = from_utf8(&expanded_s).unwrap();
  canonicalize(Path::new(&expanded)).with_context(|| format!("Couldn't get true path of {expanded:?}"))
}
