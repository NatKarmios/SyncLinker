mod cli;
mod config;
mod ctx;
mod sync;
mod util;

use anyhow::{Context, Result};
use ctx::{ARGS, CONFIG};
use notify::{event::EventKind, Watcher};
use std::path::Path;
use sync::{clean, sync};
use util::get_path;

fn initial_sync() -> Result<()> {
    for (to_s, folder) in CONFIG.folders.iter() {
        let to = get_path(to_s)?;
        clean(&to)?;
        for from_s in folder.inputs.iter() {
            let from = get_path(from_s)?;
            sync(&from, &to)?;
        }
    }
    Ok(())
}

fn clean_and_sync<P: AsRef<Path>>(from: P, to: P) -> Result<()> {
    clean(&to)?;
    sync(&from, &to)?;
    Ok(())
}

fn on_watch_event<P: AsRef<Path>>(r: notify::Result<notify::Event>, from: P, to: P) {
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
        let to = get_path(to_s)?;
        for from_s in folder.inputs.iter() {
            let from = get_path(from_s)?;
            let from_ = from.clone();
            let to_ = to.clone();
            let mut watcher = notify::recommended_watcher(move |r: notify::Result<notify::Event>| {
                on_watch_event(r, &from_, &to_)
            })
            .with_context(|| format!("Couldn't create watcher for {from:?}"))?;
            watcher
                .watch(&from, notify::RecursiveMode::NonRecursive)
                .with_context(|| format!("Couldn't create watcher for {from:?}"))?;
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
