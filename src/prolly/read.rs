#[cfg(feature = "serde")]
use serde::de::DeserializeOwned;
use {
    crate::{
        prolly::node::{AsNode, LeafMeta, Node, Pos},
        storage::StorageRead,
        Error,
    },
    std::borrow::Borrow,
};
/// TODO: possibly nuke the root ownership, - depending on what design the parent maps/tables desire.
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
    R: DeserializeOwned + AsNode,
{
    pub fn get_leaf<Q>(&mut self, k: &Q) -> Result<Option<Vec<(R::K, R::V)>>, Error>
    where
        Q: PartialOrd,
        R::K: DeserializeOwned + Clone + PartialOrd + Borrow<Q>,
        R::V: DeserializeOwned + Clone,
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
        match root.as_node() {
            Node::Branch(block) => {
                let (item_i, item) = match block
                    .iter()
                    .take_while(|(item_k, _)| item_k.borrow() <= k)
                    .enumerate()
                    .last()
                {
                    Some(t) => t,
                    None => return Ok(None),
                };
                let mut buf = Vec::new();
                self.storage.read(item.1.as_ref(), &mut buf)?;
                let node: Node<R::K, R::V> = serde_json::from_slice(&buf)?;
                recur_leaf(self.storage, k, node)
            }
            Node::Leaf(block) => Ok(Some(block.clone())),
        }
    }
    // block.iter().map(|(_, v)| v).cloned().collect::<Vec<_>>(),
}
#[cfg(all(feature = "serde", feature = "serde_json"))]
fn recur_leaf<S, Q, K, V>(
    storage: &S,
    k: &Q,
    node: Node<K, V>,
) -> Result<Option<Vec<(K, V)>>, Error>
where
    S: StorageRead,
    K: DeserializeOwned + PartialOrd + Borrow<Q>,
    V: DeserializeOwned,
    Q: PartialOrd,
{
    match node {
        Node::Branch(block) => {
            let item = match block
                .iter()
                .take_while(|(item_k, _)| item_k.borrow() <= k)
                .last()
            {
                Some(item) => item,
                None => return Ok(None),
            };
            let mut buf = Vec::new();
            storage.read(item.1.as_ref(), &mut buf)?;
            let node: Node<K, V> = serde_json::from_slice(&buf)?;
            recur_leaf(storage, k, node)
        }
        Node::Leaf(block) => Ok(Some(block)),
    }
}
#[cfg(test)]
pub mod test {
    use {
        super::*,
        crate::{
            prolly::{create::CreateTree, roller::Config as RollerConfig},
            storage::Memory,
        },
    };
    const DEFAULT_PATTERN: u32 = (1 << 8) - 1;
    #[test]
    fn get_leaf() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        let storage = Memory::new();
        let kvs = (0..61).map(|i| (i, i * 10)).collect::<Vec<_>>();
        let addr = {
            let mut tree =
                CreateTree::with_roller(&storage, RollerConfig::with_pattern(DEFAULT_PATTERN));
            for &(k, v) in kvs.iter() {
                tree = tree.push(k, v).unwrap();
            }
            let addr = dbg!(tree.commit().unwrap().unwrap());
            dbg!(&storage);
            addr
        };
        let mut tree = Tree::<'_, _, _, Node<u32, u32>>::new(&storage, addr);
        dbg!(tree.get_leaf(&0));
        dbg!(tree.get_leaf(&1));
        dbg!(tree.get_leaf(&2));
        for (expected_k, _expected_v) in kvs {
            assert!(tree
                .get_leaf(&expected_k)
                .unwrap()
                .unwrap()
                .into_iter()
                .find(|&(k, _)| k == expected_k)
                .is_some());
        }
    }
}
