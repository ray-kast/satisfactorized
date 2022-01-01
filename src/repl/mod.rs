mod command;
mod readline;

use command::BaseCommand;
use readline::Editor;

use crate::prelude::*;

pub fn run() {
    let mut rl = Editor::new();

    loop {
        use rustyline::error::ReadlineError;

        let line = match rl.readline("» ") {
            Ok(l) => l,
            Err(ReadlineError::Eof) => break,
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

        handle_cmd(cmd)
            .map_err(|e| error!("Command failed: {:?}", e))
            .ok();
    }
}

fn handle_cmd(cmd: BaseCommand) -> Result {
    use BaseCommand::*;

    match cmd {
        Want(i, n) => {},
        Calculate => {},
    }

    Ok(())
}
