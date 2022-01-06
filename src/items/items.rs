use crate::HashMap;

pub struct RemoveStackError {}

pub struct ItemRegistry(HashMap<String, Item>, HashMap<Item, String>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemStack(Item, Amount);

impl From<(Item, Amount)> for ItemStack {
    fn from((item, amt): (Item, Amount)) -> Self { Self(item, amt) }
}

impl From<ItemStack> for (Item, Amount) {
    fn from(ItemStack(item, amt): ItemStack) -> Self { (item, amt) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStacks(HashMap<Item, Amount>);

impl ItemStacks {
    fn insert_one(&mut self, ItemStack(item, amt): ItemStack) -> bool {
        use std::collections::hash_map::Entry;

        match self.0.entry(item) {
            Entry::Vacant(v) if amt != Amount::ZERO => {
                v.insert(amt);
                true
            },
            Entry::Occupied(mut m) if m.get() + amt != *m.get() => {
                *m.get_mut() += amt;
                true
            },
            _ => false,
        }
    }

    fn insert_many(&mut self, stacks: ItemStacks) {
        for stack in stacks.0 {
            self.insert_one(stack.into());
        }
    }

    fn try_remove_one(&mut self, stack: ItemStack) -> Result<(), RemoveStackError> {}
}
