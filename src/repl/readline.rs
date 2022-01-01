use std::ops::{Deref, DerefMut};

use dispose::{Disposable, Dispose};

use crate::prelude::*;

type Rl = rustyline::Editor<()>;
pub struct Editor(Rl);

impl Editor {
    const HISTFILE: &'static str = ".satis-hist";

    pub fn new() -> Disposable<Self> {
        let mut rl = Rl::new();

        rl.load_history(Self::HISTFILE)
            .map_err(|e| error!("Failed to load history: {:?}", e))
            .ok();

        Disposable::new(Self(rl))
    }
}

impl Deref for Editor {
    type Target = Rl;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for Editor {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl Dispose for Editor {
    fn dispose(mut self) {
        self.0
            .save_history(Self::HISTFILE)
            .map_err(|e| error!("Failed to save history: {:?}", e))
            .ok();
    }
}
