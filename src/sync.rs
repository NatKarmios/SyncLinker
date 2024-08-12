use std::{
    fs::{self, canonicalize},
    path::Path,
};

use anyhow::{Context, Result};

use crate::ctx::ARGS;

macro_rules! log {
  ($($tts:tt)*) => {
    if !ARGS.quiet {
      println!("{}{}", if ARGS.dry_run { "[DRY RUN] " } else { "" }, format!($($tts)*));
    };
  }
}

/// Symlinks everything in `from` in `to`
///
/// Errors if paths are invalid, or if a non-symlink would be overwritten
pub fn sync<P: AsRef<Path>>(from: P, to: P) -> Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();
    let read_dir =
        fs::read_dir(from).with_context(|| format!("Couldn't read directory {from:?}"))?;
    for file in read_dir {
        let source_file = file?;
        let source = source_file.path();
        let dest = to.join(source_file.file_name());
        let (should_delete, should_link) = match (dest.exists(), dest.is_symlink()) {
            (true, false) => {
                // Non-symlink file
                eprintln!("WARNING: '{dest:?}' exists and is not a symlink");
                (false, false)
            }
            (true, true) => {
                // Existing symlink
                let link = dest
                    .read_link()
                    .with_context(|| format!("Couldn't read link {dest:?}"))?;
                if link == source {
                    // Skip
                    (false, false)
                } else {
                    // Replace
                    (true, true)
                }
            }
            (false, true) => (true, false),  // Broken symlink
            (false, false) => (false, true), // No file
        };
        if should_delete && !ARGS.dry_run {
            fs::remove_file(&dest)?
        }
        if should_link {
            log!("{source:?} -> {dest:?}");
            if !ARGS.dry_run {
                symlink::symlink_auto(&source, &dest)?
            };
        }
    }
    Ok(())
}

/// Removes dead symlinks in `path`
pub fn clean<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    let path = canonicalize(path).with_context(|| format!("Couldn't get true path of {path:?}"))?;
    let read_dir = fs::read_dir(&path).with_context(|| format!("Couldn't read dir {path:?}"))?;
    for file in read_dir {
        let file = file?;
        let path: Box<Path> = file.path().into();
        if !path.exists() {
            log!("Removing {}", path.display());
            if !ARGS.dry_run {
                fs::remove_file(&path)?
            };
        }
    }
    Ok(())
}
