pub mod refimpl;
use crate::{
    core::deser::{Deser, Error as DeserError},
    value::{Addr, Value},
};
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HashKey([u8; 32]);
impl HashKey {
    /// Hash the provided bytes and create an `Addr` of the bytes.
    pub fn hash<B: AsRef<[u8]>>(bytes: B) -> Self {
        Self(Addr::hash(bytes).into_inner())
    }
}
/// A node within an [Unordered Set](crate::primitive::unordered_set).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[derive(Debug, Clone)]
pub enum Node {
    Branch(Vec<(HashKey, Addr)>),
    Leaf(Vec<(HashKey, Value)>),
}
impl Node {
    /// Push the given `KeyValue` into the `KeyValues`.
    ///
    /// # Panics
    ///
    /// If the variants are not aligned between this instance and what is being pushed
    /// this code will panic.
    pub fn push(&mut self, item: NodeItem) {
        match (self, item) {
            (Self::Branch(ref mut v), NodeItem::Branch(item)) => v.push(item),
            (Self::Leaf(ref mut v), NodeItem::Leaf(item)) => v.push(item),
            (_, _) => panic!("NodeItem pushed to unaligned Node enum vec"),
        }
    }
    /// Return the first key of the underlying branch or leaf.
    pub fn first_key(&self) -> Option<&HashKey> {
        match self {
            Self::Branch(v) => v.first().map(|(k, _)| k),
            Self::Leaf(v) => v.first().map(|(k, _)| k),
        }
    }
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Branch(v) => v.is_empty(),
            Self::Leaf(v) => v.is_empty(),
        }
    }
    pub fn len(&self) -> usize {
        match self {
            Self::Branch(v) => v.len(),
            Self::Leaf(v) => v.len(),
        }
    }
}
impl IntoIterator for Node {
    type Item = NodeItem;
    type IntoIter = Box<dyn Iterator<Item = NodeItem> + Send>;
    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Branch(v) => Box::new(v.into_iter().map(NodeItem::Branch)),
            Self::Leaf(v) => Box::new(v.into_iter().map(NodeItem::Leaf)),
        }
    }
}
pub enum NodeItem {
    Branch((HashKey, Addr)),
    Leaf((HashKey, Value)),
}
impl NodeItem {
    pub fn serialize_inner(&self, deser: &Deser) -> Result<Vec<u8>, DeserError> {
        match self {
            Self::Branch((_, value)) => deser.to_vec(value),
            Self::Leaf((_, value)) => deser.to_vec(value),
        }
    }
}
