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
    /// A initial measure to limit the size of the buffer, based on a simple
    /// len measurement.
    ///
    /// Eventually this will be changed to some sort of measured heapsize.
    buffer_len_limit: usize,
    state: LevelState<'s, S, K,V>,
}
impl<'s, S, K, V> Level<'s, S, K, V> {
    pub fn new(buffer_len_limit: usize) -> Self {
Self{
    buffer_len_limit,
    state: LevelState::Leaf{
        block_buffer: Vec::new(),
    }
}
    }
    pub fn insert(&mut self, k: K, v: Option<V>) {
self.state.push(k,v);
    }
    pub fn flush(self) -> Result<Self, Error> {
        // TODO: include Addr, somehow
        todo!()
    }
}
struct Block<K, V> {
    block: Vec<(K, V)>,
}
enum LevelState<'s, S, K, V> {
    Branch {
        child: Box<Level<'s, S, K, V>>,
        block_buffer: Vec<(K, Addr)>,
    },
    Leaf {
        block_buffer: Vec<(K, Option<V>)>,
    },
}
impl<'s, S, K, V> LevelState<'s, S, K, V> {
    pub fn push(&mut self, k: K, v: Option<V>) {
        match &mut self.state {
            LevelState::Branch{
                child,
                ..
            } => {
child.push(k,v);
                }
                Self::Leaf{block_buffer} => {
                    block_buffer.push(k,v);
                }
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
        let mut update = Tree::<'_, _, _, u32, u32>::new(&storage, addr);
        update.insert(3, 30);
        dbg!(update.commit::<Node<_, _>>());
    }
}
