pub mod refimpl;
use crate::{
    deser::{Deser, Error as DeserError, Serialize},
    value::{Addr, Value},
};
/// An alias to a [`Node`] with owned parameters.
pub type NodeOwned = Node<Value, Addr>;
/// A node within a [Prolly List](crate::primitive::prollylist).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
pub enum Node<Value, Addr> {
    Branch(Vec<Addr>),
    Leaf(Vec<Value>),
}
impl<V, A> Node<V, A> {
    /// Push the given `NodeItem` into the `Node`.
    ///
    /// # Panics
    ///
    /// If the variants are not aligned between this instance and what is being pushed
    /// this code will panic.
    pub fn push(&mut self, item: NodeItem<V, A>) {
        match (self, item) {
            (Self::Branch(ref mut v), NodeItem::Branch(item)) => v.push(item),
            (Self::Leaf(ref mut v), NodeItem::Leaf(item)) => v.push(item),
            (_, _) => panic!("NodeItem pushed to unaligned Node enum vec"),
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
impl<V, A> IntoIterator for Node<V, A>
where
    V: Send + 'static,
    A: Send + 'static,
{
    type Item = NodeItem<V, A>;
    type IntoIter = Box<dyn Iterator<Item = NodeItem<V, A>> + Send>;
    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Branch(v) => Box::new(v.into_iter().map(NodeItem::Branch)),
            Self::Leaf(v) => Box::new(v.into_iter().map(NodeItem::Leaf)),
        }
    }
}
pub enum NodeItem<Value, Addr> {
    Branch(Addr),
    Leaf(Value),
}
impl<V, A> NodeItem<V, A>
where
    V: Serialize,
    A: Serialize,
{
    pub fn serialize_inner(&self, deser: &Deser) -> Result<Vec<u8>, DeserError> {
        match self {
            Self::Branch(item) => deser.serialize(item),
            Self::Leaf(item) => deser.serialize(item),
        }
    }
}
