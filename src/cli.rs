use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub struct Opts {
    #[clap(short, long, default_value = "config.yaml")]
    pub config: PathBuf,
}
