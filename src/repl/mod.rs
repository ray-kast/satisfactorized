mod command;
mod readline;

use command::BaseCommand;
use readline::Editor;

use crate::{cli::Opts, config::Config, prelude::*};

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

        if let Ok(next) =
            handle_cmd(cmd, state.clone()).map_err(|e| error!("Command failed: {:?}", e))
        {
            state = next;
        }
    }
}

#[derive(Clone)]
struct State {
    want: (),
}

impl State {
    fn new() -> Self { Self { want: () } }
}

fn handle_cmd(cmd: BaseCommand, mut state: State) -> Result<State> {
    use BaseCommand::*;

    match cmd {
        Want(item, amt) => {},
        Unwant(item, amt) => {},
        Calculate => {},
    }

    Ok(state)
}
