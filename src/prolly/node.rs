use crate::Addr;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
pub trait RootNode {
    type K;
    type V;
    fn node(&self) -> &Node<Self::K, Self::V>;
}
/// The embed-friendly tree data structure, representing the root of the tree in either
/// values or `Ref<Addr>`s.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub enum Node<K, V> {
    Branch(Vec<(K, Addr)>),
    Leaf(Vec<(K, V)>),
}
impl<K, V> Node<K, V> {
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
impl<K, V> RootNode for Node<K, V> {
    type K = K;
    type V = V;
    fn node(&self) -> &Self {
        &self
    }
}
