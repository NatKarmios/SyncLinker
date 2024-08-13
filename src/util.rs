use std::{
    fs::canonicalize,
    path::{Path, PathBuf},
    str::from_utf8,
};

use anyhow::{Context, Result};
use tilde_expand::tilde_expand;

pub fn get_path(path: &str) -> Result<PathBuf> {
    let expanded_s = tilde_expand(path.as_bytes());
    let expanded = from_utf8(&expanded_s).unwrap();
    canonicalize(Path::new(&expanded))
        .with_context(|| format!("Couldn't get true path of {expanded:?}"))
}

pub fn check_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    if !path.is_dir() {
        anyhow::bail!("{path:?} is not a directory");
    }
    Ok(())
}

pub fn get_dir(path_s: &str) -> Result<PathBuf> {
    let path = get_path(path_s)?;
    check_dir(&path)?;
    Ok(path)
}
