#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The lowest storage block within Fixity, a Node within a Prolly Tree.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
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
    /// Len of the underlying vec.
    pub fn len(&self) -> usize {
        match self {
            Self::Branch(v) => v.len(),
            Self::Leaf(v) => v.len(),
        }
    }
}
