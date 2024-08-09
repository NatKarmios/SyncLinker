use clap::Parser;
use lazy_static::lazy_static;
use std::path::Path;

use crate::cli::Args;
use crate::config::Config;

lazy_static! {
    pub static ref ARGS: Args = Args::parse();
    pub static ref CONFIG: Config = {
        let config_path = Path::new(&ARGS.config);
        Config::load(config_path).unwrap()
    };
}
