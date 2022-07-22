use std::mem;

use fixity_store::{
    deser::Deserialize,
    store::{Repr, StoreError},
};

// TODO: move .. somewhere? Not sure if Store or Structs..

#[derive(Clone, PartialEq, Eq)]
pub enum OwnedOrRepr<T, D> {
    Owned(T),
    Repr(Repr<T, D>),
}
impl<T, D> Default for OwnedOrRepr<T, D>
where
    T: Default,
{
    fn default() -> Self {
        Self::Owned(T::default())
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Oor<T, D>(OwnedOrReprInvalid<T, D>);
impl<T, D> Oor<T, D> {
    pub fn inner(&self) -> &OwnedOrRepr<T, D> {
        match &self.0 {
            OwnedOrReprInvalid::Oor(oor) => &oor,
            OwnedOrReprInvalid::Invalid => {
                unreachable!("OwnedOrReprInvalid::Invalid reached")
            },
        }
    }
    pub fn owned_as_mut(&mut self) -> Result<&mut T, StoreError>
    where
        T: Deserialize<D>,
    {
        let (new_inner, repr_to_owned_res) =
            match mem::replace(&mut self.0, OwnedOrReprInvalid::Invalid) {
                inner @ OwnedOrReprInvalid::Oor(OwnedOrRepr::Owned(_)) => (inner, Ok(())),
                OwnedOrReprInvalid::Oor(OwnedOrRepr::Repr(repr)) => match repr.repr_to_owned() {
                    Ok(owned) => (OwnedOrReprInvalid::Oor(OwnedOrRepr::Owned(owned)), Ok(())),
                    Err(err) => (OwnedOrReprInvalid::Oor(OwnedOrRepr::Repr(repr)), Err(err)),
                },
                OwnedOrReprInvalid::Invalid => {
                    unreachable!("OwnedOrReprInvalid::Invalid reached")
                },
            };
        self.0 = new_inner;
        match repr_to_owned_res {
            Ok(()) => match &mut self.0 {
                OwnedOrReprInvalid::Oor(OwnedOrRepr::Owned(t)) => Ok(t),
                OwnedOrReprInvalid::Oor(OwnedOrRepr::Repr(_)) => {
                    unreachable!("Repr variant persisted despite above return")
                },
                OwnedOrReprInvalid::Invalid => {
                    unreachable!("OwnedOrReprInvalid::Invalid reached")
                },
            },
            Err(err) => Err(err),
        }
    }
}
impl<T, D> Default for Oor<T, D>
where
    T: Default,
{
    fn default() -> Self {
        Self(OwnedOrReprInvalid::Oor(OwnedOrRepr::default()))
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum OwnedOrReprInvalid<T, D> {
    Oor(OwnedOrRepr<T, D>),
    Invalid,
}
impl<T, D> Default for OwnedOrReprInvalid<T, D>
where
    T: Default,
{
    fn default() -> Self {
        Self::Oor(OwnedOrRepr::default())
    }
}
