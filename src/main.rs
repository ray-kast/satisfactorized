#![feature(hash_drain_filter)]

mod atom;
mod cli;
mod config;
mod items;
mod repl;

pub type HashMap<K, V> = std::collections::HashMap<K, V, ahash::RandomState>;
pub type HashSet<K> = std::collections::HashSet<K, ahash::RandomState>;

pub mod prelude {
    pub use anyhow::{anyhow, bail, Context, Error};
    pub use docbot::prelude::*;
    pub use log::{debug, error, info, trace, warn};

    pub use crate::atom::{Atom, Memoized};

    pub type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;
}

fn main() {
    use std::process;

    use clap::Parser;
    use cli::Opts;

    let mut b = env_logger::Builder::from_default_env();
    b.filter_module("rustyline", log::LevelFilter::Info);
    b.init();

    let opts = Opts::parse();

    match repl::run(opts) {
        Ok(()) => (),
        Err(e) => {
            log::error!("Fatal error: {:?}", e);
            process::exit(-1);
        },
    }
}
