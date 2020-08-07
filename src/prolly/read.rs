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
#[cfg(all(feature = "cjson", feature = "serde"))]
impl<'s, S, A, R> Tree<'s, S, A, R>
where
    S: StorageRead,
    R: DeserializeOwned + RootNode,
{
    pub fn get<Q>(&mut self, _k: &Q) -> R::V
    where
        R::K: Borrow<Q>,
    {
        todo!()
    }
}
fn recur_get<S, Q, K, V>(storage: &S, k: &Q, node: &Node<K, V>) -> Result<Option<Node<K, V>>, Error>
where
    S: Storage,
    K: Borrow<Q>,
{
    match node {
        Node::Branch(block) => {
            // foo
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
        let mut tree =
            CreateTree::with_roller(&storage, RollerConfig::with_pattern(DEFAULT_PATTERN));
        for item in (0..61).map(|i| (i, i * 10)) {
            tree = tree.push(item).unwrap();
        }
        dbg!(tree.commit().unwrap());
        dbg!(&storage);
    }
}
