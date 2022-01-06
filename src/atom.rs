use std::{fmt, hash::Hash, marker::PhantomData};

use parking_lot::RwLock;
use rand::prelude::*;

use crate::HashMap;

pub trait Memoized: From<Atom<Self>> + Into<Atom<Self>> + Copy + Eq + Hash + 'static {
    type Value;

    fn registry_ref() -> &'static Registry<Self>;
}

#[repr(transparent)]
#[derive(PartialEq, Eq, Hash)]
pub struct Atom<T>(usize, PhantomData<T>);

impl<T> Clone for Atom<T> {
    fn clone(&self) -> Self { Self(self.0, PhantomData::default()) }
}

impl<T> Copy for Atom<T> {}

impl<T: Memoized> fmt::Display for Atom<T>
where T::Value: fmt::Display + Clone + Eq + Hash
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        T::registry_ref()
            .peek_value((*self).into(), |v| v.fmt(f))
            .unwrap()
    }
}

impl<T: Memoized> fmt::Debug for Atom<T>
where T::Value: fmt::Debug + Clone + Eq + Hash
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::registry_ref()
            .peek_value((*self).into(), |v| {
                f.debug_tuple("Atom").field(&self.0).field(v).finish()
            })
            .unwrap()
    }
}

#[allow(missing_debug_implementations)]
pub struct Registry<T: Memoized>(RwLock<RegistryInner<T>>);

pub struct RegistryInner<T: Memoized> {
    gen: StdRng,
    fwd: HashMap<T::Value, Atom<T>>,
    rev: HashMap<Atom<T>, T::Value>,
}

impl<T: Memoized> Registry<T> {
    pub fn new() -> Self {
        Self(RwLock::new(RegistryInner {
            gen: StdRng::from_entropy(),
            fwd: HashMap::default(),
            rev: HashMap::default(),
        }))
    }
}

impl<T: Memoized> Default for Registry<T> {
    fn default() -> Self { Self::new() }
}

impl<T: Memoized> Registry<T>
where T::Value: Clone + Eq + Hash
{
    pub fn register(&self, values: impl IntoIterator<Item = T::Value>) {
        use std::collections::hash_map::Entry;

        let RegistryInner {
            ref mut gen,
            ref mut fwd,
            ref mut rev,
        } = *self.0.write();

        for val in values {
            let fwd = if let Entry::Vacant(v) = fwd.entry(val.clone()) {
                v
            } else {
                continue;
            };

            loop {
                let id = Atom(gen.gen(), PhantomData::default());

                if let Entry::Vacant(v) = rev.entry(id) {
                    v.insert(val);
                    fwd.insert(id);
                    break;
                }
            }
        }
    }

    pub fn memoize<Q: ?Sized + Hash + Eq>(&self, val: &Q) -> Option<T>
    where T::Value: std::borrow::Borrow<Q> {
        let inner = self.0.read();
        inner.fwd.get(val).copied().map(Into::into)
    }

    #[must_use = "The return value indicates whether the closure was run"]
    pub fn peek_value<U>(&self, id: T, f: impl FnOnce(&T::Value) -> U) -> Option<U> {
        let inner = self.0.read();

        inner.rev.get(&id.into()).map(f)
    }
}

#[macro_export]
macro_rules! easy_atom {
    ($ty:ident, $value:ty, $err_ty:ty, | $inval:ident | $err:expr) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $ty(crate::atom::Atom<$ty>);

        impl $ty {
            pub fn new<
                Q: ?Sized + Eq + ::std::hash::Hash + ::std::borrow::ToOwned<Owned = $value>,
            >(
                val: &Q,
            ) -> Result<Self, $err_ty>
            where $value: ::std::borrow::Borrow<Q> {
                crate::atom::Memoized::registry_ref()
                    .memoize(val)
                    .ok_or_else(|| {
                        let $inval = val.to_owned();
                        $err
                    })
            }

            pub fn cloned(&self) -> $value {
                crate::atom::Memoized::registry_ref()
                    .peek_value(*self, Clone::clone)
                    .unwrap()
            }
        }

        impl crate::atom::Memoized for $ty {
            type Value = $value;

            fn registry_ref() -> &'static crate::atom::Registry<$ty> {
                lazy_static::lazy_static! {
                    static ref REGISTRY: crate::atom::Registry<$ty> = crate::atom::Registry::new();
                }

                &*REGISTRY
            }
        }

        impl From<crate::atom::Atom<$ty>> for $ty {
            #[inline]
            fn from(a: crate::atom::Atom<$ty>) -> Self { Self(a) }
        }

        impl From<$ty> for crate::atom::Atom<$ty> {
            #[inline]
            fn from(t: $ty) -> Self { t.0 }
        }

        impl TryFrom<$value> for $ty {
            type Error = $err_ty;

            fn try_from(val: $value) -> Result<$ty, $err_ty> { Self::new(&val) }
        }

        impl From<$ty> for $value {
            fn from(t: $ty) -> Self { t.cloned() }
        }

        impl ::std::fmt::Display for $ty
        where $value: ::std::fmt::Display
        {
            #[inline]
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result { self.0.fmt(f) }
        }

        impl ::std::str::FromStr for $ty
        where $value: ::std::borrow::Borrow<str>
        {
            type Err = $err_ty;

            fn from_str(s: &str) -> Result<Self, $err_ty> { Self::new(s) }
        }
    };
}
