// pub mod cursor_create;
// pub mod cursor_read;
// pub mod cursor_update;
// pub mod lru_read;
pub mod refimpl;
pub mod roller;
// cursor_create::CursorCreate,
// cursor_read::CursorRead,
// lru_read::LruRead,
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use {
    crate::{
        value::{Addr, KeyOwned, ValueOwned},
        Error,
    },
    rkyv::Archive,
};
pub(crate) const ONE_LEN_BLOCK_WARNING: &str =
    "writing key & value that exceeds block size, this is highly inefficient";
/// An alias to a [`Node`] with owned parameters.
pub type NodeOwned = Node<Vec<(KeyOwned, Addr)>, Vec<(ValueOwned, Addr)>>;
/// The lowest storage block within Fixity, a Node within a Prolly Tree.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[derive(Debug, Archive, rkyv::Serialize, rkyv::Deserialize, Eq, PartialEq)]
pub enum Node<B, L> {
    Branch(B),
    Leaf(L),
}
/*
use std::ops::Deref;
pub enum NodeBL<B, L> {
    Branch(B),
    Leaf(L),
}
impl<B, L> NodeBL<B, L> {
    /// Return the key for this whole node, aka the first element's key.
    pub fn key<'a, K: 'a, V: 'a, A: 'a>(&'a self) -> Option<&K>
    where
        B: Deref<Target = [(K, V)]>,
        L: Deref<Target = [(K, A)]>,
    {
        match self {
            Self::Branch(v) => v.get(0).map(|(k, _)| k),
            Self::Leaf(v) => v.get(0).map(|(k, _)| k),
        }
    }
    /// Len of the underlying vec.
    pub fn len(&self) -> usize {
        match self {
            Self::Branch(v) => v.len(),
            Self::Leaf(v) => v.len(),
        }
    }
    /// Whether or not the underlying vec is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Branch(v) => v.is_empty(),
            Self::Leaf(v) => v.is_empty(),
        }
    }
}
*/
// Disabled for compat while sussing.
// impl NodeOwned {
//     /// Consume self and return the key for this whole node, aka the first element's key.
//     pub fn into_key(self) -> Option<KeyOwned> {
//         match self {
//             Self::Branch(mut v) => {
//                 if v.is_empty() {
//                     None
//                 } else {
//                     Some(v.swap_remove(0).0)
//                 }
//             },
//             Self::Leaf(mut v) => {
//                 if v.is_empty() {
//                     None
//                 } else {
//                     Some(v.swap_remove(0).0)
//                 }
//             },
//         }
//     }
// }
