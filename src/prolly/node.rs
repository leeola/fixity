#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The lowest storage block within Fixity, a Node within a Prolly Tree.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub enum Node<Key, Value, Addr, Meta> {
    BranchWithMeta {
        meta: Meta,
        addrs: Vec<(Key, Addr)>,
    },
    LeafWithMeta {
        meta: Meta,
        values: Vec<(Key, Value)>,
    },
    Branch(Vec<(Key, Addr)>),
    Leaf(Vec<(Key, Value)>),
}
impl<K, V, A, M> Node<K, V, A, M> {
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
// TODO: Maybe deprecate meta and pos
#[derive(Debug)]
pub struct LeafMeta<K, V> {
    pos: Pos,
    leaf: Vec<(K, V)>,
}
#[derive(Debug, Default, Copy, Clone)]
pub struct Pos {
    x: usize,
    y: usize,
}
impl Pos {
    pub fn with_x(x: usize) -> Self {
        Self { x, y: 0 }
    }
    pub fn add_x(&self, x: usize) -> Self {
        Self {
            x: self.x + x,
            y: self.y,
        }
    }
    pub fn add_y(&self, y: usize) -> Self {
        Self {
            x: self.x,
            y: self.y + y,
        }
    }
}
