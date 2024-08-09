use clap::Parser;

/// Merges & syncs folders via symlinks
#[derive(Parser)]
#[command(about)]
pub struct Args {
    /// Quiet mode
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// Dry run
    #[arg(short, long, default_value_t = false)]
    pub dry_run: bool,

    /// Don't watch folders, only run once
    #[arg(short, long, default_value_t = false)]
    pub once: bool,

    /// Config file location
    #[arg(long, default_value = "./config.yaml")]
    pub config: String,
}
