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
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::{
    path::{Path, PathBuf},
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
};
use sync::{clean, sync};
use util::{check_dir, get_dir};

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
    let mut error_logged = false;
    loop {
        match (check_dir(&from), check_dir(&to)) {
            (Ok(()), Ok(())) => return true,
            (Err(e), _) | (_, Err(e)) => {
                if !error_logged {
                    log::info!("{e}");
                    error_logged = true;
                }
                if !should_retry(&e, settings) {
                    return false;
                }
            }
        }
    }
}

fn clean_and_sync<P: AsRef<Path>>(from: P, to: P) -> Result<()> {
    clean(&to)?;
    sync(&from, &to)?;
    Ok(())
}

fn on_watch_event(r: notify::Result<notify::Event>, snd: &Sender<()>) {
    match r {
        Ok(event) => match event.kind {
            EventKind::Access(_) => (),
            _ => snd.send(()).unwrap(),
        },
        Err(e) => log::error!("Watch error: {:?}", e),
    };
}

// type WatcherBox = (Box<dyn Watcher>, Arc<()>);
struct WatcherBox {
    _watcher: Box<dyn Watcher>,
    dropped: Arc<AtomicBool>,
}

impl Drop for WatcherBox {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::Relaxed);
    }
}

fn get_watcher<P: AsRef<Path>>(
    from: P,
    to: P,
    settings: &Settings,
) -> Result<(WatcherBox, Receiver<()>)> {
    let from = from.as_ref();
    let to = to.as_ref();
    let (snd, rcv): (Sender<()>, Receiver<()>) = std::sync::mpsc::channel();
    let snd_ = snd.clone();
    let mut watcher = notify::recommended_watcher(move |r: notify::Result<notify::Event>| {
        on_watch_event(r, &snd)
    })
    .with_context(|| format!("Couldn't watch {from:?}"))?;
    watcher
        .watch(from.as_ref(), notify::RecursiveMode::NonRecursive)
        .with_context(|| format!("Couldn't watch {from:?}"))?;
    let dropped = Arc::new(AtomicBool::new(false));
    let watcher_box = WatcherBox {
        _watcher: Box::new(watcher),
        dropped: dropped.clone(),
    };

    // Manually checking that dirs exist in case of unmount
    // https://github.com/notify-rs/notify/issues/627
    if let Some(liveness_interval) = settings.liveness_interval {
        let from = from.to_owned();
        let to = to.to_owned();
        thread::spawn(move || {
            while !dropped.load(Ordering::Relaxed) {
                log::trace!("Liveness: checking {from:?} -> {to:?}");
                thread::sleep(Duration::from_secs(liveness_interval));

                match (check_dir(&from), check_dir(&to)) {
                    (Ok(()), Ok(())) => (),
                    (Err(_), _) => {
                        log::trace!("Liveness: {from:?} is dead!");
                        let _ = snd_.send(());
                    }
                    (_, Err(_)) => {
                        log::trace!("Liveness: {to:?} is dead!");
                        let _ = snd_.send(());
                    }
                };
            }
            log::trace!("Liveness: stopped checking {from:?} -> {to:?}");
        });
    }
    Ok((watcher_box, rcv))
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
            thread::sleep(Duration::from_secs(settings.retry_delay));
            true
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
            let (_w, rcv) = get_watcher(&from, &to, &settings).unwrap();
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
