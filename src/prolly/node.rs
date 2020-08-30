use crate::Addr;
#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Deserialize, Serialize};
pub trait AsNode {
    type K: DeserializeOwned;
    type V: DeserializeOwned;
    fn as_node(&self) -> &Node<Self::K, Self::V>;
}
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub enum NodeC<Key, Value, Meta, Addr> {
    RootBranch {
        meta: Meta,
        addrs: Vec<(Key, Addr)>,
    },
    Branch(Vec<(Key, Addr)>),
    RootLeaf {
        meta: Meta,
        addrs: Vec<(Key, Value)>,
    },
    Leaf(Vec<(Key, Value)>),
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
impl<K, V> AsNode for Node<K, V>
where
    K: DeserializeOwned,
    V: DeserializeOwned,
{
    type K = K;
    type V = V;
    fn as_node(&self) -> &Self {
        &self
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
#[cfg(test)]
pub mod test {
    use {
        super::*,
        crate::{
            prolly::{create::CreateTree, node::Node, roller::Config as RollerConfig},
            storage::Memory,
        },
    };
    #[test]
    fn deserialize_ref() {
        let mut buf = vec![vec![
            r#"
{
    "Root",
}
                    "#,
        ]];
    }
    fn impl_deserialize_node_ref<'de, C>(mut node_bufs: &mut Vec<Vec<u8>>) -> Vec<NodeC<C>>
    where
        C: ContainerRef<'de>,
    {
        let mut nodes = Vec::new();
        for buf in node_bufs.iter() {
            let node: NodeC<C> = serde_json::from_slice(&buf).unwrap();
            nodes.push(node);
            nodes.push(node);
        }
        nodes
    }
}
