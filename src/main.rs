mod cli;
mod repl;

pub mod prelude {
    pub use anyhow::{anyhow, bail, Context, Error};
    pub use docbot::prelude::*;
    pub use log::{debug, error, info, trace, warn};

    pub type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;
}

fn main() {
    use clap::Parser;
    use cli::Opts;

    let mut b = env_logger::Builder::from_default_env();
    b.filter_module("rustyline", log::LevelFilter::Info);
    b.init();

    let Opts { config } = Opts::parse();

    let _ = config;

    repl::run();
}
