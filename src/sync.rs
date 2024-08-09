use std::{fs, path::Path};

use anyhow::{Context, Result};

use crate::ctx::ARGS;

/// Symlinks everything in `from` in `to`
///
/// Errors if paths are invalid, or if a non-symlink would be overwritten
pub fn sync(from: &Path, to: &Path) -> Result<()> {
  let read_dir = fs::read_dir(from).with_context(|| format!("Couldn't read {}", from.display()))?;
  for file in read_dir {
    let source = file.with_context(|| format!("IO error while reading {}" , from.display()))?;
    let dest: Box<Path> = to.join(source.file_name()).into();
    let (should_delete, should_link) = match (dest.exists(), dest.is_symlink()) {
      (true, false) => {
        // Non-symlink file
        eprintln!("WARNING: '{}' exists and is not a symlink", dest.display());
        (false, false)
      },
      (true, true) => {
        // Existing symlink
        let link = dest.read_link().with_context(|| format!("Couldn't read link {}", dest.display()))?;
        if link == source.path() {
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
      fs::remove_file(&dest).with_context(|| format!("Couldn't remove {}", dest.display()))?;
    }
    if should_link {
      symlink::symlink_auto(&source.path(), &dest).with_context(|| format!("Couldn't symlink {} to {}", source.path().display(), dest.display()))?;
      if !ARGS.quiet { println!("'{}' -> '{}'", source.path().display(), dest.display()) }
    }
  }
  Ok(())
}

/// Removes dead symlinks in `path`
pub fn clean(path: &Path) -> Result<()> {
  let read_dir = fs::read_dir(path).with_context(|| format!("Couldn't read {}", path.display()))?;
  for file in read_dir {
    let file = file.with_context(|| format!("IO error while reading {}" , path.display()))?;
    let path: Box<Path> = file.path().into();
    if !path.exists() {
      fs::remove_file(&path).with_context(|| format!("Couldn't remove {}", path.display()))?;
      if !ARGS.quiet { println!("Removing '{}'", path.display()) }
    }
  };
  Ok(())
}