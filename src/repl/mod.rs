mod command;
mod readline;

use command::BaseCommand;
use num_traits::identities::one;
use readline::Editor;

use crate::{
    cli::Opts,
    config::Config,
    items::{ItemStack, ItemStacks},
    prelude::*,
};

pub fn run(opts: Opts) -> Result<()> {
    let Opts { config } = opts;
    let config = Config::load(config).context("Failed to load config")?;

    debug!("Loaded config: {:#?}", config);

    let mut rl = Editor::new();
    let mut state = State::new();

    loop {
        use rustyline::error::ReadlineError;

        let line = match rl.readline("» ") {
            Ok(l) => l,
            Err(ReadlineError::Eof) => break Ok(()),
            Err(ReadlineError::Interrupted) => continue,
            Err(e) => {
                error!("Failed to read user input: {:?}", e);
                continue;
            },
        };

        trace!("> {:?}", line);

        let cmd = match command::parse_str(&line) {
            Ok(cmd) => {
                rl.add_history_entry(line);

                trace!("{:?}", cmd);

                cmd
            },
            Err(err) => {
                if !matches!(err, docbot::CommandParseError::NoInput) {
                    rl.add_history_entry(line);
                }

                trace!("{:?}", err);
                let err = command::FormatError.fold_command_parse(err).unwrap();

                if !err.is_empty() {
                    eprintln!("{}", err);
                }

                continue;
            },
        };

        if let Ok(next) = state
            .clone()
            .handle_cmd(cmd)
            .map_err(|e| error!("Command failed: {:?}", e))
        {
            state = next;
        }
    }
}

#[derive(Debug, Clone)]
struct State {
    want: ItemStacks,
}

impl State {
    fn new() -> Self {
        Self {
            want: ItemStacks::empty(),
        }
    }

    fn handle_cmd(mut self, cmd: BaseCommand) -> Result<Self> {
        match cmd {
            BaseCommand::Want(item, amt) => self.want += ItemStack(item, amt.unwrap_or_else(one)),
            // TODO: add and use a saturating_remove for this
            BaseCommand::Unwant(item, amt) => self
                .want
                .try_remove_one(ItemStack(item, amt.unwrap_or_else(one)))
                .context("Failed to remove item from wanted list")?,
            BaseCommand::Show => println!("{:#?}", self),
            BaseCommand::Calculate => todo!(),
        }

        Ok(self)
    }
}
