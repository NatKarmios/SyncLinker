use clap::Parser;
use lazy_static::lazy_static;

use crate::cli::Args;
use crate::config::Config;
use crate::util::get_path;

lazy_static! {
    pub static ref ARGS: Args = Args::parse();
    pub static ref CONFIG: Config = {
        let config_path = get_path(&ARGS.config).unwrap();
        Config::load(&config_path).unwrap()
    };
}
