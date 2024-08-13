use std::fmt::Display;

use clap::{Parser, ValueEnum};
use log::LevelFilter;

#[derive(Clone, Debug, ValueEnum)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn to_level_filter(&self) -> LevelFilter {
        match self {
            LogLevel::Off => LevelFilter::Off,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{self:?}").to_lowercase())
    }
}

/// Merges & syncs folders via symlinks
#[derive(Parser)]
#[command(about)]
pub struct Args {
    #[arg(short, long, default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,

    #[arg(short, long, default_value_t = false)]
    pub dry_run: bool,

    /// Don't watch folders, only run once
    #[arg(short, long, default_value_t = false)]
    pub once: bool,

    /// Config file location
    #[arg(long, default_value = "./config.yaml")]
    pub config: String,
}
