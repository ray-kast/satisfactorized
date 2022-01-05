use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub struct Opts {
    #[clap(short, long, default_value = "config.yml")]
    pub config: PathBuf,
}
