mod cli;
mod config;
mod ctx;
mod sync;

use anyhow::{Context, Result};
use ctx::{ARGS, CONFIG};
use notify::{event::EventKind, Watcher};
use std::path::Path;
use sync::{clean, sync};

fn initial_sync() -> Result<()> {
    for (to_s, folder) in CONFIG.folders.iter() {
        let to = Path::new(to_s);
        clean(to)?;
        for from_s in folder.inputs.iter() {
            let from = Path::new(from_s);
            sync(from, to)?;
        }
    }
    Ok(())
}

fn clean_and_sync(from: &Path, to: &Path) -> Result<()> {
    clean(to)?;
    sync(from, to)?;
    Ok(())
}

fn on_watch_event(r: notify::Result<notify::Event>, from: &Path, to: &Path) {
    match r {
        Ok(event) => match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                clean_and_sync(from, to).unwrap_or_else(|e| eprintln!("Error: {}", e));
            }
            _ => (),
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
            })
            .with_context(|| format!("Couldn't create watcher for {}", from.display()))?;
            watcher.watch(from, notify::RecursiveMode::NonRecursive)?;
            watchers.push(Box::new(watcher) as Box<dyn Watcher>);
        }
    }
    Ok(watchers)
}

fn run() -> Result<()> {
    initial_sync()?;
    if !ARGS.once {
        // Naming watchers, as they get killed if they're dropped
        let _watchers = start_watch()?;
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
