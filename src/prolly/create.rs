#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use {
    crate::{
        prolly::roller::{Config as RollerConfig, Roller},
        storage::StorageWrite,
        Addr, Error,
    },
    multibase::Base,
    std::mem,
};
pub struct Create<'s, S, Key, Value> {
    storage: &'s S,
    roller_config: RollerConfig,
    leaf: LeafCursor<'s, S, Key, Value>,
}
impl<'s, S, K, V> Create<'s, S, K, V> {
    pub fn new(storage: &'s S) -> Self {
        Self::with_roller(storage, RollerConfig::default())
    }
    pub fn with_roller(storage: &'s S, roller_config: RollerConfig) -> Self {
        Self {
            storage,
            roller_config,
            leaf: Leaf::new(storage, Roller::with_config(roller_config)),
        }
    }
}
struct Leaf<'s, S, Key, Value> {
    storage: &'s S,
    roller: Roller,
}
impl<'s, S, K, V> Leaf<'s, S, K, V> {
    pub fn new(storage: &'s S, roller: Roller) -> Self {
        Self { storage, roller }
    }
}
impl<'s, S, K, V> Create<'s, S, K, V>
where
    S: StorageWrite,
    K: std::fmt::Debug + Serialize + Clone,
    V: std::fmt::Debug + Serialize,
{
    pub fn flush(self) -> Result<Option<Node<K, V>>, Error> {
        // self.root.flush()
        todo!("flush")
    }
    /// Flush this `Create` and write the results to the internal storage,
    /// consuming `Self`.
    ///
    /// This is useful for low level interactions with the tree who only want to store
    /// a node without any root header data.
    ///
    /// # Errors
    /// Fails if flush, serialization or calls to storage fail.
    pub fn commit(self) -> Result<Option<Addr>, Error> {
        // let node = match self.root.flush()? {
        //     Some(node) => node,
        //     None => return Ok(None),
        // };
        // let node_bytes = cjson::to_vec(&node).map_err(|err| format!("{:?}", err))?;
        // let node_addr = {
        //     let node_hash = <[u8; 32]>::from(blake3::hash(&node_bytes));
        //     multibase::encode(Base::Base58Btc, &node_hash)
        // };
        // self.storage
        //     .write(&node_addr, &*node_bytes)
        //     .map_err(|err| format!("{:?}", err))?;
        // Ok(Some(node_addr.into()))
        todo!()
    }
    pub fn push(self, k: K, v: V) -> Result<Self, Error> {
        let Self {
            storage,
            roller_config,
            mut root,
        } = self;
        let mut node_opt = root.push(k, v)?;
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
// struct BranchCursor<'s, S, K, V> {
//     storage: &'s S,
//     roller: Roller,
// }
/*
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
                if block.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(Node::Branch(block)))
                }
            }
            LevelState::Leaf { block } => {
                if block.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(Node::Leaf(block)))
                }
            }
        }
    }
    pub fn push(&mut self, k: K, v: V) -> Result<Option<Node<K, V>>, Error> {
        match &mut self.block {
            LevelState::Branch { child, block } => {
                let node = match child.push(k, v)? {
                    Some(node) => node,
                    None => return Ok(None),
                };
                Self::push_branch_kv(&self.storage, &mut self.roller, block, node)
            }
            LevelState::Leaf { block } => {
                let elm = (k, v);
                let boundary = self
                    .roller
                    .roll_bytes(&cjson::to_vec(&elm).map_err(|err| format!("{:?}", err))?);
                block.push(elm);
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
*/
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
        // let storage = Memory::new();
        // let mut tree =
        //     CreateTree::with_roller(&storage, RollerConfig::with_pattern(DEFAULT_PATTERN));
        // for (k, v) in (0..61).map(|i| (i, i * 10)) {
        //     tree = tree.push(k, v).unwrap();
        // }
        // dbg!(tree.flush());
        // dbg!(&storage);
    }
}
