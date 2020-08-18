#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use {
    crate::{
        prolly::{
            create::CreateTree,
            node::{AsNode, Node},
            read::Tree as ReadTree,
            roller::{Config as RollerConfig, Roller},
        },
        storage::{Storage, StorageRead, StorageWrite},
        Addr, Error,
    },
    multibase::Base,
    std::{cmp::Eq, collections::HashMap, hash::Hash},
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Update<V> {
    Insert(V),
    Remove,
}
pub struct Tree<'s, S, A, K, V> {
    storage: &'s S,
    addr: A,
    updates: HashMap<K, Update<V>>,
}
impl<'s, S, A, K, V> Tree<'s, S, A, K, V>
where
    // S: StorageWrite,
    K: std::fmt::Debug + Eq + Hash,
{
    pub fn new(storage: &'s S, addr: A) -> Self {
        Self {
            storage,
            addr,
            updates: HashMap::new(),
        }
    }
    pub fn insert(&mut self, k: K, v: V) {
        self.updates.insert(k, Update::Insert(v));
    }
    pub fn remove(&mut self, k: K) {
        self.updates.insert(k, Update::Remove);
    }
}
impl<'s, S, A, K, V> Tree<'s, S, A, K, V>
where
    S: StorageRead + StorageWrite,
    A: Clone,
    K: std::fmt::Debug + DeserializeOwned + Serialize + Clone,
    V: std::fmt::Debug + DeserializeOwned + Serialize,
{
    fn flush_ret_storage<R>(self) -> Result<(S, Option<Node<K, V>>), Error>
    where
        R: DeserializeOwned + AsNode<K = K, V = V>,
    {
        // let Self {
        //     storage,
        //     addr,
        //     updates,
        // } = self;
        // let reader = ReadTree::<'_, _, _, R>::new(&storage, addr);
        // let create = CreateTree::<'_, _, K, V>::new(&storage);
        todo!("flush update")
    }
    pub fn flush<R>(&mut self) -> Result<Option<Addr>, Error>
    where
        R: DeserializeOwned + AsNode<K = K, V = V>,
    {
        let reader = ReadTree::<'_, _, _, R>::new(&self.storage, &self.addr);
        let create = CreateTree::<'_, _, K, V>::new(&self.storage);
        todo!("flush")
    }
    pub fn commit<R>(self) -> Result<Option<Addr>, Error>
    where
        R: DeserializeOwned + AsNode<K = K, V = V>,
    {
        todo!("commit")
        // let (storage, node) = match self.flush_ret_storage::<R>()? {
        //     (s, Some(node)) => (s, node),
        //     (_, None) => return Ok(None),
        // };
        // let node_bytes = cjson::to_vec(&node)?;
        // let node_addr = {
        //     let node_hash = <[u8; 32]>::from(blake3::hash(&node_bytes));
        //     multibase::encode(Base::Base58Btc, &node_hash)
        // };
        // storage.write(&node_addr, &*node_bytes)?;
        // Ok(Some(node_addr.into()))
    }
}
struct Level<'s, S, K, V> {
    state: LevelState<'s, S, K, V>,
    // cursor, but not stored - exists on insert.
    // window: Vec<(K,V)>,
    // roller: Roller,
}
impl<'s, S, K, V> Level<'s, S, K, V> {
    pub fn new(storage: &'s S) -> Self {
        Self {
            state: LevelState::Leaf {
                storage,
                window: Vec::new(),
            },
        }
    }
    pub fn flush(self) -> Result<Self, Error> {
        // TODO: include Addr, somehow
        todo!()
    }
}
impl<'s, S, K, V> Level<'s, S, K, V>
where
    K: Eq + Ord,
{
    pub fn insert(&mut self, k: K, v: Option<V>) -> Option<(K, Addr)> {
        // general level behavior:
        // - find the node (addr) on the tree, at this level, that the key belongs in.
        // - if that's the same node (addr) as before, do nothing.
        // - if it's a new addr, the old level state needs to be "cleaned up"
        //      - this cleaning involes attempting to write cached block.
        //      - if the block is cleanly written, ie no remaining elements, do nothing.
        //      - if the block has remaining elements, either from a partial write or
        //          failure to find a boundary at all, expand the level state and repeat.

        // how do we know to expand the window?
        //
        // - if the key moves past the window, we'd have to try and flush the window.
        //   - if it comes back clean, load a new window.
        //   - if it comes back dirty, expand the window and check if the key is now within the
        //      window and repeat the entire process.
        self.state.insert(k, v);
    }
}
struct Block<K, V> {
    block: Vec<(K, V)>,
}
enum LevelState<'s, S, K, V> {
    Branch {
        storage: &'s S,
        child: Box<Level<'s, S, K, V>>,
        window: Vec<(K, Addr)>,
    },
    Leaf {
        storage: &'s S,
        window: Vec<(K, V)>,
    },
}
struct Leaf<K, V> {
    window: Vec<(K, V)>,
}
enum InsertResult<K, V> {
    WindowMutated,
    WindowDangling((K, V)),
    WindowClosed((K, V)),
}
enum WindowMut<K> {
    LeftSideSlid((K, Addr)),
    NoBorderFound,
    Closed((K, Addr)),
}
impl<'s, S, K, V> LevelState<'s, S, K, V>
where
    K: Eq + Ord,
{
    pub fn expand_window(&mut self, node: Node<K, V>) -> Option<(K, Addr)> {
        todo!()
    }
    pub fn insert(&mut self, k: K, v: Option<V>) {
        match self {
            LevelState::Branch { child, .. } => {
                child.insert(k, v);
                todo!("branch insert")
            }
            Self::Leaf { window, .. } => {
                use std::cmp::Ordering;
                enum KeyIndex {
                    Exists(usize),
                    Before(usize),
                    End,
                }
                let change = window
                    .iter()
                    .enumerate()
                    .find_map(|(i, kv)| match kv.0.cmp(&k) {
                        Ordering::Less => None,
                        Ordering::Equal => Some(KeyIndex::Exists(i)),
                        Ordering::Greater => Some(KeyIndex::Before(i)),
                    })
                    .unwrap_or(KeyIndex::End);
                match (change, v) {
                    (KeyIndex::Exists(i), Some(v)) => {
                        window[i] = (k, v);
                    }
                    (KeyIndex::Exists(i), None) => {
                        window.remove(i);
                    }
                    (KeyIndex::Before(i), Some(v)) => {
                        window.insert(i, (k, v));
                    }
                    // the caller is trying to remove a key not found in the index.
                    (KeyIndex::Before(i), None) => {}
                    (KeyIndex::End, Some(v)) => {
                        window.push((k, v));
                    }
                    // the caller is trying to remove something _after_ the leaf, but it
                    // doesn't exist.
                    //
                    // The parent `Level` should have expanded the window to always insert
                    // within the window, so this should never happen i think..
                    (KeyIndex::End, None) => {}
                }
            }
        }
    }
    // pub fn move_window
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
    const DEFAULT_PATTERN: u32 = (1 << 8) - 1;
    #[test]
    fn insert() {
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
            tree = tree.push(1, 10).unwrap();
            tree = tree.push(2, 20).unwrap();
            let addr = tree.commit().unwrap().unwrap();
            dbg!(&storage);
            addr
        };
        let mut update = Tree::new(&storage, 5);
        update.insert(3, 30);
        // dbg!(update.commit::<Node<_, _>>());
    }
}
