use std::{fmt, fmt::Write};

use crate::prelude::*;

#[derive(Docbot, Debug)]
pub enum BaseCommand {
    /// `(want|add) <item> [amount]`
    /// Add an item to the selected outputs
    ///
    /// # Arguments
    /// item: The name of the item
    /// amount: The amount requested (default 1)
    Want(String, Option<f64>),

    /// `calculate`
    /// Compute the build strategy for the currently selected outputs
    Calculate,
}

type Formatted = Result<String, fmt::Error>;
pub struct FormatError;

impl FormatError {
    fn write_options<S: AsRef<str>>(
        s: &mut String,
        opts: impl IntoIterator<Item = S>,
    ) -> fmt::Result {
        opts.into_iter().enumerate().try_for_each(|(i, opt)| {
            if i != 0 {
                write!(s, ", ")?;
            }

            write!(s, "'{}'", opt.as_ref())
        })
    }
}

impl FoldError for FormatError {
    type Output = Formatted;

    fn no_id_match(&self, given: String, available: &'static [&'static str]) -> Formatted {
        let mut s = String::new();

        write!(
            s,
            "Not sure what you mean by {:?}.  Available options are: ",
            given
        )?;

        Self::write_options(&mut s, available)?;

        Ok(s)
    }

    fn ambiguous_id(&self, possible: &'static [&'static str], given: String) -> Formatted {
        let mut s = String::new();

        write!(s, "Not sure what you mean by {:?}.  Could be: ", given)?;

        Self::write_options(&mut s, possible)?;

        Ok(s)
    }

    fn no_input(&self) -> Formatted { Ok(String::new()) }

    fn missing_required(&self, cmd: &'static str, arg: &'static str) -> Formatted {
        Ok(format!(
            "Missing required argument '{}' to command '{}'",
            arg, cmd
        ))
    }

    fn bad_convert(&self, cmd: &'static str, arg: &'static str, inner: Formatted) -> Formatted {
        Ok(format!(
            "Couldn't parse argument '{}' of command '{}': {}",
            arg, cmd, inner?
        ))
    }

    fn trailing(&self, cmd: &'static str, extra: String) -> Formatted {
        Ok(format!(
            "Unexpected extra argument {:?} to '{}'",
            extra, cmd
        ))
    }

    fn subcommand(&self, subcmd: &'static str, inner: Formatted) -> Formatted {
        Ok(format!("Subcommand '{}' failed: {:?}", subcmd, inner))
    }

    fn other(&self, error: docbot::Anyhow) -> Formatted {
        Ok(format!("Unexpected error: {:?}", error))
    }
}

pub fn parse_str(s: impl AsRef<str>) -> Result<BaseCommand, docbot::CommandParseError> {
    BaseCommand::parse(docbot::tokenize_str_simple(s.as_ref()))
}
