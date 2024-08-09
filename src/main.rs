mod config;
mod sync;
mod cli;
mod ctx;

use std::path::Path;
use anyhow::{Context, Result};
use notify::{Watcher, event::EventKind};
use sync::{sync, clean};
use ctx::{ARGS, CONFIG};


fn initial_sync() -> Result<()> {
  for (to_s, folder) in CONFIG.folders.iter() {
    let to = Path::new(to_s);
    clean(to)?;
    for from_s in folder.inputs.iter() {
      let from = Path::new(from_s);
      sync(from, to)?;
    };
  };
  Ok(())
}

fn clean_and_sync(from: &Path, to: &Path) -> Result<()> {
  clean(to)?;
  sync(from, to)?;
  Ok(())
}

fn on_watch_event(r: notify::Result<notify::Event>, from: &Path, to: &Path) {
  match r {
    Ok(event) => {
      match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
          clean_and_sync(from, to).unwrap_or_else(|e| eprintln!("Error: {}", e));
        },
        _ => (),
      }
    },
    Err(e) => eprintln!("Watch error: {:?}", e),
  };
}

fn start_watch() -> Result<Vec<Box<dyn Watcher>>> {
  let mut watchers = Vec::new();
  for (to_s, folder) in CONFIG.folders.iter() {
    let to = Path::new(to_s);
    for from_s in folder.inputs.iter() {
      let from = Path::new(from_s);
      let mut watcher = notify::recommended_watcher(|r: notify::Result<notify::Event>| {
        on_watch_event(r, from, to)
      }).with_context(|| format!("Couldn't create watcher for {}", from.display()))?;
      watcher.watch(from, notify::RecursiveMode::NonRecursive)?;
      watchers.push(Box::new(watcher) as Box<dyn Watcher>);
    }
  }
  Ok(watchers)
}

fn run() -> Result<()> {
  initial_sync()?;
  if !ARGS.once {
    start_watch()?;
    loop {
      std::thread::park();
    }
  }
  Ok(())
}

fn main() {
  match run() {
    Ok(()) => (),
    Err(e) => eprintln!("Error: {}", e),
  }
}
