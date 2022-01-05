use thiserror::Error;

use super::Amount;
use crate::{easy_atom, prelude::*, HashMap};

#[derive(Debug, Error)]
#[error("Unrecognized item name {0:?}")]
pub struct InvalidItem(String);

easy_atom!(Item, String, InvalidItem, |name| InvalidItem(name));

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemStack(pub Item, pub Amount);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStacks(HashMap<Item, Amount>);

impl ItemStacks {
    pub fn new(mut stacks: HashMap<Item, Amount>) -> Self {
        stacks.drain_filter(|_, v| *v == Amount::new(0.0).unwrap());

        Self(stacks)
    }
}
