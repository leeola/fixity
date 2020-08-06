#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use {
    crate::{
        prolly::roller::{Config as RollerConfig, Roller},
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
/// The primary constructor implementation to distribute values
pub struct Tree<'s, S, K, V> {
    storage: &'s S,
    roller_config: RollerConfig,
    root: Level<'s, S, K, V>,
}
impl<'s, S, K, V> Tree<'s, S, K, V> {
    pub fn new(storage: &'s S) -> Self {
        Self::with_roller(storage, RollerConfig::default())
    }
    pub fn with_roller(storage: &'s S, roller_config: RollerConfig) -> Self {
        Self {
            storage,
            roller_config,
            root: Level::new(storage, Roller::with_config(roller_config)),
        }
    }
}
#[cfg(all(feature = "cjson", feature = "serde"))]
impl<'s, S, K, V> Tree<'s, S, K, V>
where
    S: StorageWrite,
    K: std::fmt::Debug + Serialize + Clone,
    V: std::fmt::Debug + Serialize,
{
    pub fn flush(self) -> Result<Option<Node<K, V>>, Error> {
        self.root.flush()
    }
    pub fn push(self, kv: (K, V)) -> Result<Self, Error> {
        let Self {
            storage,
            roller_config,
            mut root,
        } = self;
        let mut node_opt = root.push(kv)?;
        while let Some(node) = node_opt {
            let mut roller = Roller::with_config(roller_config);
            let child = Box::new(root);
            let mut block = Vec::new();
            node_opt = Level::push_branch_kv(storage, &mut roller, &mut block, node)?;
            root = Level {
                storage,
                roller,
                block: LevelState::Branch { child, block },
            }
        }
        Ok(Self {
            storage,
            roller_config,
            root,
        })
    }
}
struct Level<'s, S, K, V> {
    storage: &'s S,
    roller: Roller,
    block: LevelState<'s, S, K, V>,
}
impl<'s, S, K, V> Level<'s, S, K, V> {
    pub fn new(storage: &'s S, roller: Roller) -> Self {
        Self {
            storage,
            roller,
            block: LevelState::Leaf { block: Vec::new() },
        }
    }
}
#[cfg(all(feature = "cjson", feature = "serde"))]
impl<'s, S, K, V> Level<'s, S, K, V>
where
    S: StorageWrite,
    K: std::fmt::Debug + Serialize + Clone,
    V: std::fmt::Debug + Serialize,
{
    pub fn with_child(storage: &'s S, roller: Roller, child: Box<Self>) -> Self {
        Self {
            storage,
            roller,
            block: LevelState::Branch {
                child,
                block: Vec::new(),
            },
        }
    }
    #[must_use]
    #[inline(always)]
    fn push_branch_kv(
        storage: &S,
        roller: &mut Roller,
        block: &mut Vec<(K, Addr)>,
        node: Node<K, V>,
    ) -> Result<Option<Node<K, V>>, Error> {
        let k = node
            .key()
            .ok_or_else(|| Error::from("child level returned empty node"))?
            .clone();
        let child_node_addr = {
            // if this child returns a node, we have a new key to track in *this* node.
            // So hash the child node, so store the key with the hash of the child node.
            let node_bytes = cjson::to_vec(&node).map_err(|err| format!("{:?}", err))?;
            let node_addr = {
                let node_hash = <[u8; 32]>::from(blake3::hash(&node_bytes));
                multibase::encode(Base::Base58Btc, &node_hash)
            };
            storage
                .write(&node_addr, &*node_bytes)
                .map_err(|err| format!("{:?}", err))?;
            node_addr
        };
        let kv = (k, Addr::from(child_node_addr));
        let boundary = roller.roll_bytes(&cjson::to_vec(&kv).map_err(|err| format!("{:?}", err))?);
        block.push(kv);
        if boundary {
            Ok(Some(Node::Branch(mem::replace(block, Vec::new()))))
        } else {
            Ok(None)
        }
    }
    pub fn flush(self) -> Result<Option<Node<K, V>>, Error> {
        match self.block {
            LevelState::Branch { child, mut block } => {
                if let Some(node) = child.flush()? {
                    let k = node
                        .key()
                        .ok_or_else(|| Error::from("child level returned empty node"))?
                        .clone();
                    let child_node_addr = {
                        let node_bytes =
                            cjson::to_vec(&node).map_err(|err| format!("{:?}", err))?;
                        let node_addr = {
                            let node_hash = <[u8; 32]>::from(blake3::hash(&node_bytes));
                            multibase::encode(Base::Base58Btc, &node_hash)
                        };
                        self.storage
                            .write(&node_addr, &*node_bytes)
                            .map_err(|err| format!("{:?}", err))?;
                        node_addr
                    };
                    block.push((k, Addr::from(child_node_addr)));
                };
                Ok(Some(Node::Branch(block)))
            }
            LevelState::Leaf { block } => Ok(Some(Node::Leaf(block))),
        }
    }
    pub fn push(&mut self, kv: (K, V)) -> Result<Option<Node<K, V>>, Error> {
        match &mut self.block {
            LevelState::Branch { child, block } => {
                let node = match child.push(kv)? {
                    Some(node) => node,
                    None => return Ok(None),
                };
                Self::push_branch_kv(&self.storage, &mut self.roller, block, node)
            }
            LevelState::Leaf { block } => {
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
enum LevelState<'s, S, K, V> {
    Branch {
        child: Box<Level<'s, S, K, V>>,
        block: Vec<(K, Addr)>,
    },
    Leaf {
        block: Vec<(K, V)>,
    },
}
#[cfg(test)]
pub mod test {
    use {super::*, crate::storage::Memory};
    const DEFAULT_PATTERN: u32 = (1 << 8) - 1;
    #[test]
    fn poc() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        let storage = Memory::new();
        let mut tree = Tree::with_roller(
            &storage,
            RollerConfig {
                pattern: dbg!(DEFAULT_PATTERN),
                window_size: 67,
            },
        );
        for item in (0..30).map(|i| (i, i * 10)) {
            tree = tree.push(item).unwrap();
        }
        dbg!(tree.flush());
        dbg!(&storage);
    }
}
