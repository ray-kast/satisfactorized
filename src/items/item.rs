use std::ops::{Add, AddAssign, Deref, Sub, SubAssign};

use num_traits::identities::Zero;
use thiserror::Error;

use super::Amount;
use crate::{easy_atom, prelude::*, HashMap};

#[derive(Debug, Error)]
#[error("Unrecognized item name {0:?}")]
pub struct InvalidItem(String);

easy_atom!(Item, String, InvalidItem, |name| InvalidItem(name));

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemStack(pub Item, pub Amount);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStacks(HashMap<Item, Amount>);

#[derive(Debug, Error)]
#[error("A provided stack was too large to remove")]
pub struct StackTooLarge;

impl ItemStacks {
    pub fn new(mut stacks: HashMap<Item, Amount>) -> Self {
        stacks.drain_filter(|_, v| *v == Amount::new(0.0).unwrap());

        Self(stacks)
    }

    pub fn empty() -> Self { Self(HashMap::default()) }

    pub fn stacks_iter(&self) -> impl Iterator<Item = ItemStack> + '_ {
        self.iter().map(|(i, a)| ItemStack(*i, *a))
    }

    pub fn insert_one(&mut self, stack: ItemStack) {
        self.0
            .entry(stack.0)
            .and_modify(|a| *a += stack.1)
            .or_insert(stack.1);
    }

    pub fn insert(&mut self, stacks: &ItemStacks) {
        stacks.stacks_iter().for_each(|s| self.insert_one(s));
    }

    pub fn try_remove_one(&mut self, stack: ItemStack) -> Result<(), StackTooLarge> {
        if stack.1.is_zero() {
            return Ok(());
        }

        let amt = self
            .0
            .get_mut(&stack.0)
            .and_then(|r| {
                r.checked_sub(stack.1).map(|s| {
                    *r = s;
                    s
                })
            })
            .ok_or(StackTooLarge)?;

        if amt == Amount::zero() {
            self.0.remove(&stack.0);
        }

        Ok(())
    }

    pub fn try_remove(&mut self, stacks: &ItemStacks) -> Result<(), StackTooLarge> {
        let tmp = stacks
            .stacks_iter()
            .map(|stack| {
                self.0
                    .get(&stack.0)
                    .and_then(|r| r.checked_sub(stack.1))
                    .ok_or(StackTooLarge)
                    .map(|v| (stack.0, v))
            })
            .collect::<Result<Vec<_>, _>>()?;

        self.0.extend(tmp.into_iter());

        Ok(())
    }

    fn with(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}

impl Deref for ItemStacks {
    type Target = HashMap<Item, Amount>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl AddAssign<ItemStack> for ItemStacks {
    fn add_assign(&mut self, rhs: ItemStack) { self.insert_one(rhs) }
}

impl AddAssign<&ItemStacks> for ItemStacks {
    fn add_assign(&mut self, rhs: &ItemStacks) { self.insert(rhs) }
}

impl SubAssign<ItemStack> for ItemStacks {
    fn sub_assign(&mut self, rhs: ItemStack) { self.try_remove_one(rhs).unwrap() }
}

impl SubAssign<&ItemStacks> for ItemStacks {
    fn sub_assign(&mut self, rhs: &ItemStacks) { self.try_remove(rhs).unwrap() }
}

impl Add<ItemStack> for ItemStacks {
    type Output = ItemStacks;

    fn add(self, rhs: ItemStack) -> Self { self.with(|s| *s += rhs) }
}

impl Sub<ItemStack> for ItemStacks {
    type Output = ItemStacks;

    fn sub(self, rhs: ItemStack) -> Self { self.with(|s| *s += rhs) }
}

macro_rules! auto_impls {
    (< > $ty:ty, $i:ty, $op:ident, $fn:ident, $op_asn:ident, $fn_asn:ident) => {
        impl $op<$ty> for $i {
            type Output = $ty;

            fn $fn(self, rhs: $ty) -> $ty { rhs.$fn(self) }
        }

        impl $op<$ty> for &$i {
            type Output = $ty;

            fn $fn(self, rhs: $ty) -> $ty { rhs.$fn(self) }
        }

        auto_impls!($ty, $i, $op, $fn, $op_asn, $fn_asn);
    };

    ($ty:ty, $i:ty, $op:ident, $fn:ident, $op_asn:ident, $fn_asn:ident) => {
        impl $op<&$i> for $ty {
            type Output = $ty;

            fn $fn(self, rhs: &$i) -> $ty { self.$fn(*rhs) }
        }

        impl $op_asn<&$i> for $ty {
            fn $fn_asn(&mut self, rhs: &$i) { self.$fn_asn(*rhs) }
        }
    };
}

auto_impls!(<> ItemStacks, ItemStack, Add, add, AddAssign, add_assign);
auto_impls!(ItemStacks, ItemStack, Sub, sub, SubAssign, sub_assign);
