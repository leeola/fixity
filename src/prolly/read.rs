#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use {
    crate::{
        prolly::{
            node::{Node, RootNode},
            roller::{Config as RollerConfig, Roller},
        },
        storage::{Storage, StorageRead, StorageWrite},
        Addr, Error,
    },
    multibase::Base,
    std::{borrow::Borrow, collections::HashMap, mem},
};
pub struct Tree<'s, S, A, R> {
    storage: &'s S,
    addr: A,
    root: Option<R>,
}
impl<'s, S, A, R> Tree<'s, S, A, R> {
    pub fn new(storage: &'s S, addr: A) -> Self {
        Self {
            storage,
            addr,
            root: None,
        }
    }
}
#[cfg(all(feature = "serde", feature = "serde_json"))]
impl<'s, S, A, R> Tree<'s, S, A, R>
where
    S: StorageRead,
    A: AsRef<str>,
    R: std::fmt::Debug + DeserializeOwned + RootNode,
{
    pub fn get_leaf<Q>(&mut self, k: &Q) -> Result<Option<Vec<R::V>>, Error>
    where
        Q: PartialOrd,
        R::K: PartialOrd + Borrow<Q>,
    {
        let root = match &self.root {
            Some(root) => root,
            None => {
                let mut buf = Vec::new();
                self.storage.read(self.addr.as_ref(), &mut buf)?;
                let root: R = serde_json::from_slice(&buf)?;
                self.root.replace(root);
                self.root.as_ref().expect("impossibly missing")
            }
        };
        recur_get(self.storage, k, root.node())?;
        todo!("map results")
    }
}
#[cfg(all(feature = "serde", feature = "serde_json"))]
fn recur_get<S, Q, K, V>(
    storage: &S,
    k: &Q,
    node: &Node<K, V>,
) -> Result<Option<Vec<(K, V)>>, Error>
where
    S: StorageRead,
    K: PartialOrd + Borrow<Q>,
    Q: PartialOrd,
{
    match node {
        Node::Branch(block) => {
            // TODO: use iter, takewhile with a last.
            let mut working_block_item = block.get(0).unwrap();
            for item in block {
                if item.0.borrow() > k {
                    break;
                } else {
                    working_block_item = item
                }
            }
            let mut buf = Vec::new();
            storage.read(working_block_item.1.as_ref(), &mut buf)?;
            let node: Node<K, V> = serde_json::from_slice(&buf)?;
            // storage.read(
            // recur_get(storage, k, working_block_item
            todo!("branch")
        }
        Node::Leaf(block) => todo!("leaf"),
    }
}
#[cfg(test)]
pub mod test {
    use {
        super::*,
        crate::{prolly::create::CreateTree, storage::Memory},
    };
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
        let addr = {
            let mut tree =
                CreateTree::with_roller(&storage, RollerConfig::with_pattern(DEFAULT_PATTERN));
            for item in (0..61).map(|i| (i, i * 10)) {
                tree = tree.push(item).unwrap();
            }
            let addr = dbg!(tree.commit().unwrap().unwrap());
            dbg!(&storage);
            addr
        };
        let mut tree = Tree::<'_, _, _, Node<u32, u32>>::new(&storage, addr);
        dbg!(tree.get_leaf(&1));
    }
}
