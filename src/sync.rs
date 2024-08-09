use std::{ fs::{ self, canonicalize }, path::Path };

use anyhow::Result;

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
pub fn sync(from: &Path, to: &Path) -> Result<()> {
  let read_dir = fs::read_dir(from)?;
  for file in read_dir {
    let source_file = file?;
    let source_rel = source_file.path();
    let source = canonicalize(&source_rel)?;
    let dest_rel = to.join(source_file.file_name());
    let dest = canonicalize(&dest_rel)?;
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
      if !ARGS.dry_run { fs::remove_file(&dest)? };
    }
    if should_link {
      log!("{} -> {}", source_rel.display(), dest_rel.display());
      if !ARGS.dry_run { symlink::symlink_auto(&source, &dest)? };
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
      log!("Removing {}", path.display());
      if !ARGS.dry_run { fs::remove_file(&path)? };
    }
  };
  Ok(())
}