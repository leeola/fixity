#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use {
    crate::{
        prolly::RollHasher,
        storage::{Storage, StorageRead, StorageWrite},
        Addr,
    },
    multibase::Base,
    std::collections::HashMap,
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
    /// Return the first key in this node, if any.
    pub fn first_key(&self) -> Option<&K> {
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
    pub fn flush(&mut self) -> Result<Option<Node<K, V>>, Error> {
        todo!("flush")
    }
    pub fn push(&mut self, (k, v): (K, V)) -> Result<Option<Node<K, V>>, Error> {
        let keys_at_boundary: bool = match &mut self.block {
            Block::Branch { child, block } => {
                // self.handle_child_resp(child.push(item)?)
                let node = match child.push((k, v))? {
                    Some(node) => node,
                    None => return Ok(None),
                };
                // if this child returns a node, we have a new key
                // to track in *this* node. So hash the child node,
                // so store the key with the hash of the child node.
                let block_bytes = cjson::to_vec(&node).map_err(|err| format!("{:?}", err))?;

                // TODO: hash child
                // TODO: write child to storage
                let boundary = self
                    .roller
                    .roll_bytes(&cjson::to_vec(&k).map_err(|err| format!("{:?}", err))?);
                // TODO: rolling hash key
                todo!("child branch")
            }
            Block::Leaf { block } => {
                let boundary = self
                    .roller
                    .roll_bytes(&cjson::to_vec(&k).map_err(|err| format!("{:?}", err))?);
                block.push((k, v));
                boundary
            }
        };
        if keys_at_boundary {
            self.flush()
        } else {
            Ok(None)
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
