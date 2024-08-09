use std::{fs, path::{Path, absolute }};

use anyhow::Result;

use crate::ctx::ARGS;

/// Symlinks everything in `from` in `to`
///
/// Errors if paths are invalid, or if a non-symlink would be overwritten
pub fn sync(from: &Path, to: &Path) -> Result<()> {
  let read_dir = fs::read_dir(from)?;
  for file in read_dir {
    let source_file = file?;
    let source_rel = source_file.path();
    let source = absolute(&source_rel)?;
    let dest_rel = to.join(source_file.file_name());
    let dest = absolute(&dest_rel)?;
    let (should_delete, should_link) = match (dest.exists(), dest.is_symlink()) {
      (true, false) => {
        // Non-symlink file
        eprintln!("WARNING: '{}' exists and is not a symlink", dest_rel.display());
        (false, false)
      },
      (true, true) => {
        // Existing symlink
        let link = dest.read_link()?;
        if link == source {
          // Skip
          (false, false)
        } else {
          // Replace
          (true, true)
        }
      },
      (false, true) => (true, false),  // Broken symlink
      (false, false) => (false, true),  // No file
    };
    if should_delete {
      fs::remove_file(&dest)?;
    }
    if should_link {
      if !ARGS.quiet { println!("{} -> {}", source_rel.display(), dest_rel.display()) }
      symlink::symlink_auto(&source, &dest)?;
    }
  }
  Ok(())
}

/// Removes dead symlinks in `path`
pub fn clean(path: &Path) -> Result<()> {
  let read_dir = fs::read_dir(path)?;
  for file in read_dir {
    let file = file?;
    let path: Box<Path> = file.path().into();
    if !path.exists() {
      if !ARGS.quiet { println!("Removing {}", path.display()) }
      fs::remove_file(&path)?;
    }
  };
  Ok(())
}