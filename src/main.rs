mod cli;
mod config;
mod ctx;
mod sync;
mod util;

use anyhow::{Context, Result};
use config::{OnMissing, Settings};
use ctx::{ARGS, CONFIG};
use notify::{event::EventKind, Watcher};
use simplelog::SimpleLogger;
use std::{
    path::{Path, PathBuf},
    sync::mpsc::{Receiver, Sender},
    thread::{self, JoinHandle},
};
use sync::{clean, sync};
use util::{check_dir, get_dir};

fn clean_and_sync<P: AsRef<Path>>(from: P, to: P) -> Result<()> {
    clean(&to)?;
    sync(&from, &to)?;
    Ok(())
}

fn on_watch_event(r: notify::Result<notify::Event>, snd: &Sender<()>) {
    match r {
        Ok(event) => match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                snd.send(()).unwrap();
            }
            _ => (),
        },
        Err(e) => log::error!("Watch error: {:?}", e),
    };
}

fn get_watcher<P: AsRef<Path>>(p: P) -> Result<(Box<dyn Watcher>, Receiver<()>)> {
    let p = p.as_ref();
    let (snd, rcv): (Sender<()>, Receiver<()>) = std::sync::mpsc::channel();
    let snd = snd.clone();
    let mut watcher = notify::recommended_watcher(move |r: notify::Result<notify::Event>| {
        on_watch_event(r, &snd)
    })
    .with_context(|| format!("Couldn't watch {p:?}"))?;
    watcher
        .watch(p.as_ref(), notify::RecursiveMode::NonRecursive)
        .with_context(|| format!("Couldn't watch {p:?}"))?;
    Ok((Box::new(watcher), rcv))
}

fn should_retry(e: &anyhow::Error, settings: &Settings) -> bool {
    match settings.on_missing {
        OnMissing::Error => panic!("{e}"),
        OnMissing::Warn => {
            log::warn!("{e}");
            false
        }
        OnMissing::Ignore => false,
        OnMissing::Retry => {
            log::trace!("{e}; retrying in {} seconds", settings.retry_delay);
            thread::sleep(std::time::Duration::from_secs(settings.retry_delay));
            true
        }
    }
}

fn wait_and_get_dirs(from_s: &str, to_s: &str, settings: &Settings) -> Option<(PathBuf, PathBuf)> {
    let mut error_logged = false;
    loop {
        match (get_dir(from_s), get_dir(to_s)) {
            (Ok(from), Ok(to)) => return Some((from, to)),
            (Err(e), _) | (_, Err(e)) => {
                if !error_logged {
                    log::info!("{e}");
                    error_logged = true;
                }
                if !should_retry(&e, settings) {
                    return None;
                }
            }
        }
    }
}

fn are_dirs_real<P: AsRef<Path>>(from: P, to: P, settings: &Settings) -> bool {
    loop {
        match (check_dir(&from), check_dir(&to)) {
            (Ok(()), Ok(())) => return true,
            (Err(e), _) | (_, Err(e)) => {
                if !should_retry(&e, settings) {
                    return false;
                }
            }
        }
    }
}

fn watch_dir(from_s: &str, to_s: &str, settings: &Settings) -> JoinHandle<()> {
    let from_s = from_s.to_owned();
    let to_s = to_s.to_owned();
    let settings = settings.clone();
    thread::spawn(move || {
        log::trace!("Running initial sync for {from_s} -> {to_s}...");
        let (from, to) = match wait_and_get_dirs(&from_s, &to_s, &settings) {
            Some(ps) => ps,
            None => return,
        };
        clean_and_sync(&from, &to).unwrap();
        if ARGS.once {
            return;
        };
        loop {
            if !are_dirs_real(&from, &to, &settings) {
                return;
            };
            let (_w, rcv) = get_watcher(&from).unwrap();
            log::info!("Watching {from_s} for changes");
            for () in rcv.iter() {
                if !from.is_dir() || !to.is_dir() {
                    break;
                };
                clean_and_sync(&from, &to).unwrap();
            }
        }
    })
}

fn start_watches() -> Vec<JoinHandle<()>> {
    let mut threads = Vec::new();
    for (to_s, folder) in CONFIG.folders.iter() {
        let settings = &CONFIG.settings.apply_partial(&folder.settings);
        for from_s in folder.inputs.iter() {
            threads.push(watch_dir(from_s, to_s, settings));
        }
    }
    threads
}

fn main() {
    {
        let log_level = ARGS.log_level.to_level_filter();
        SimpleLogger::init(log_level, simplelog::Config::default()).unwrap();
    }
    log::set_max_level(ARGS.log_level.to_level_filter());
    log::info!(
        "==== {} version {} ====",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    let threads = start_watches();
    for thread in threads {
        thread.join().unwrap();
    }
}
