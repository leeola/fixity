use crate::{
    value::{Addr, Key, Value},
    Error,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// An alias to a [`Node`] with owned parameters.
pub type NodeOwned = Node<Key, Value, Addr>;
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
pub enum NodeType {
    Map,
}
/// The lowest storage block within Fixity, a Node within a Prolly Tree.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
pub enum Node<Key, Value, Addr> {
    Branch(Vec<(Key, Addr)>),
    Leaf(Vec<(Key, Value)>),
}
impl<K, V, A> Node<K, V, A> {
    /// Return the key for this whole node, aka the first element's key.
    pub fn key(&self) -> Option<&K> {
        match self {
            Self::Branch(v) => v.get(0).map(|(k, _)| k),
            Self::Leaf(v) => v.get(0).map(|(k, _)| k),
        }
    }
    /// Consume self and return the key for this whole node, aka the first element's key.
    pub fn into_key(self) -> Option<K> {
        match self {
            Self::Branch(mut v) => {
                if v.is_empty() {
                    None
                } else {
                    Some(v.swap_remove(0).0)
                }
            }
            Self::Leaf(mut v) => {
                if v.is_empty() {
                    None
                } else {
                    Some(v.swap_remove(0).0)
                }
            }
        }
    }
    /// Like [`Self::into_key`], but panics if called on an empty node.
    ///
    /// # Panics
    /// Panics if called on empty Node.
    pub fn into_key_unchecked(self) -> K {
        match self {
            Self::Branch(mut v) => v.swap_remove(0).0,
            Self::Leaf(mut v) => v.swap_remove(0).0,
        }
    }
    /// Len of the underlying vec.
    pub fn len(&self) -> usize {
        match self {
            Self::Branch(v) => v.len(),
            Self::Leaf(v) => v.len(),
        }
    }
}
#[cfg(feature = "borsh")]
impl<K, V, A> Node<K, V, A>
where
    K: borsh::BorshSerialize,
    V: borsh::BorshSerialize,
    A: borsh::BorshSerialize,
{
    /// Serialize and hash the Node, returning the `Addr` and bytes.
    pub fn as_bytes(&self) -> Result<(Addr, Vec<u8>), Error> {
        let bytes = crate::value::serialize(self)?;
        let addr = Addr::from(&bytes);
        Ok((addr, bytes))
    }
}
