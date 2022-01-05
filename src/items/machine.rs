use thiserror::Error;

use crate::{easy_atom, prelude::*};

#[derive(Debug, Error)]
#[error("Unrecognized machine name {0:?}")]
pub struct InvalidMachine(String);

easy_atom!(Machine, String, InvalidMachine, |name| InvalidMachine(name));
