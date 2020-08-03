#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use {
    crate::{
        prolly::RollHasher,
        storage::{Storage, StorageRead, StorageWrite},
        Addr,
    },
    multibase::Base,
    std::{collections::HashMap, mem},
};
/// A temp error type
type Error = String;
// #[cfg(test)]
const CHUNK_PATTERN: u32 = 1 << 8 - 1;
/// The embed-friendly tree data structure, representing the root of the tree in either
/// values or `Ref<Addr>`s.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
/// The primary constructor implementation to distribute values
pub struct Tree<'s, S, K, V> {
    storage: &'s S,
    depth: usize,
    width: usize,
    root: Level<'s, S, K, V>,
}
struct Level<'s, S, K, V> {
    storage: &'s S,
    roller: RollHasher,
    pattern: u32,
    block: Block<'s, S, K, V>,
}
enum Block<'s, S, K, V> {
    Branch {
        child: Box<Level<'s, S, K, V>>,
        block: Vec<(K, Addr)>,
    },
    Leaf {
        block: Vec<(K, V)>,
    },
}
impl<'s, S, K, V> Level<'s, S, K, V> {
    pub fn new(storage: &'s S) -> Self {
        let mut roller = RollHasher::new(4);
        Self {
            storage,
            pattern: CHUNK_PATTERN,
            roller,
            block: Block::Leaf { block: Vec::new() },
        }
    }
}
#[cfg(all(feature = "cjson", feature = "serde"))]
impl<'s, S, K, V> Level<'s, S, K, V>
where
    S: StorageWrite,
    K: Serialize + Clone,
    V: Serialize,
{
    pub fn flush(self) -> Result<Option<Node<K, V>>, Error> {
        match self.block {
            Block::Branch { child, block } => {
                let node = match child.flush()? {
                    Some(node) => node,
                    None => todo!("eh?"),
                };
                let k = node.key().clone();
                let child_node_addr = {
                    let node_bytes = cjson::to_vec(&node).map_err(|err| format!("{:?}", err))?;
                    let node_addr = {
                        let node_hash = <[u8; 32]>::from(blake3::hash(&node_bytes));
                        multibase::encode(Base::Base58Btc, &node_hash)
                    };
                    self.storage
                        .write(&node_addr, &*node_bytes)
                        .map_err(|err| format!("{:?}", err))?;
                    node_addr
                };
                let kv = (k, Addr::from(child_node_addr));
                let boundary = self
                    .roller
                    .roll_bytes(&cjson::to_vec(&kv).map_err(|err| format!("{:?}", err))?);
                block.push(kv);
                if boundary {
                    Ok(Some(Node::Branch(mem::replace(block, Vec::new()))))
                } else {
                    Ok(None)
                }
            }
            Block::Leaf { block } => todo!(),
        }
        todo!("flush")
    }
    pub fn push(&mut self, kv: (K, V)) -> Result<Option<Node<K, V>>, Error> {
        match &mut self.block {
            Block::Branch { child, block } => {
                let k = kv.0.clone();
                let child_node_addr = {
                    let node = match child.push(kv)? {
                        Some(node) => node,
                        None => return Ok(None),
                    };
                    // if this child returns a node, we have a new key to track in *this* node.
                    // So hash the child node, so store the key with the hash of the child node.
                    let node_bytes = cjson::to_vec(&node).map_err(|err| format!("{:?}", err))?;
                    let node_addr = {
                        let node_hash = <[u8; 32]>::from(blake3::hash(&node_bytes));
                        multibase::encode(Base::Base58Btc, &node_hash)
                    };
                    self.storage
                        .write(&node_addr, &*node_bytes)
                        .map_err(|err| format!("{:?}", err))?;
                    node_addr
                };
                let kv = (k, Addr::from(child_node_addr));
                let boundary = self
                    .roller
                    .roll_bytes(&cjson::to_vec(&kv).map_err(|err| format!("{:?}", err))?);
                block.push(kv);
                if boundary {
                    Ok(Some(Node::Branch(mem::replace(block, Vec::new()))))
                } else {
                    Ok(None)
                }
            }
            Block::Leaf { block } => {
                let boundary = self
                    .roller
                    .roll_bytes(&cjson::to_vec(&kv).map_err(|err| format!("{:?}", err))?);
                block.push(kv);
                if boundary {
                    Ok(Some(Node::Leaf(mem::replace(block, Vec::new()))))
                } else {
                    Ok(None)
                }
            }
        }
    }
}
#[cfg(test)]
pub mod test {
    use {
        super::*,
        crate::storage::{Memory, Storage, StorageRead, StorageWrite},
        maplit::hashmap,
    };
    #[test]
    fn poc() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        let storage = Memory::new();
    }
}
