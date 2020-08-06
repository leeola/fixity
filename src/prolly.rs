pub mod create;
pub mod roller;
pub mod update;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use {
    crate::{
        storage::{Storage, StorageRead, StorageWrite},
        Addr,
    },
    multibase::Base,
    std::collections::HashMap,
};
// #[cfg(test)]
const CHUNK_PATTERN: u32 = 1 << 8 - 1;
pub enum ValueType {
    Usize,
}
pub enum Node {
    Nodes(Vec<NodeRef>),
    Values(Vec<Value>),
}
pub struct NodeRef {
    key: Vec<u8>,
    addr: Vec<u8>,
}
pub struct NodeValue {
    key: Vec<u8>,
    value: Value,
}
pub enum Key {
    // Bool,
    Usize(usize),
    // String,
    // Blob,
}
impl From<usize> for Key {
    fn from(t: usize) -> Self {
        Self::Usize(t)
    }
}
pub enum Value {
    // Bool,
    Usize(usize),
    // String,
    // Blob,
    // Ref { key: Vec<u8>, addr: Vec<u8> },
}
impl From<usize> for Value {
    fn from(t: usize) -> Self {
        Self::Usize(t)
    }
}
pub struct Prolly {}
impl Prolly {
    pub fn new() -> Self {
        Self {}
    }
    pub fn commit<S>(&mut self, storage: S) -> Result<(), ()>
    where
        S: Storage,
    {
        todo!()
    }
    // pub fn new_list(&mut self) -> List<T> {
    //     todo!()
    // }
}
pub struct List {
    len: usize,
    // inserted: HashMap<usize, T>,
    appended: Vec<Value>,
}
impl List {
    pub fn new() -> Self {
        Self {
            len: 0,
            appended: Vec::new(),
        }
    }
    // pub fn commit<S>(&mut self, storage: &S) -> Result<Ref, String>
    // where
    //     S: Storage,
    // {
    //     todo!()
    // }
    pub fn append<T>(&mut self, value: T)
    where
        T: Into<Value>,
    {
        todo!()
    }
}
enum MapChange<K, V> {
    Insert((K, V)),
    Remove(K),
}
pub mod types {
    pub struct Usize(usize);
}
pub struct StagedMap<K, V> {
    changes: Vec<MapChange<K, V>>,
}
impl<K, V> StagedMap<K, V> {
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }
    // pub fn commit<S>(&mut self, storage: &S) -> Result<Ref, String>
    // where
    //     S: Storage,
    // {
    //     todo!()
    // }
    pub fn insert<T, U>(&mut self, k: T, v: U)
    where
        T: Into<K>,
        U: Into<V>,
    {
        self.changes.push(MapChange::Insert((k.into(), v.into())));
    }
}
pub struct Ref {
    ref_type: RefType,
    addr: Addr,
}
pub enum RefType {
    Map,
}
pub enum NodeItem<K, V> {
    Refs(Vec<(K, Addr)>),
    Values(Vec<(K, V)>),
}
pub struct Map<K, V> {
    items: Vec<NodeItem<K, V>>,
}
impl<K, V> Map<K, V>
where
    K: std::fmt::Debug + Serialize + Ord + Clone,
    V: Serialize,
{
    // TODO: make the map generic.
    pub fn new<S>(storage: &S, map: HashMap<K, V>) -> Result<Self, String>
    where
        S: StorageWrite,
    {
        let mut roller = RollHasher::new(4);
        let mut init_items = map.into_iter().map(|(k, v)| (k, v)).collect::<Vec<_>>();
        init_items.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
        let mut headers = Vec::new();
        let mut current_block_items = Vec::new();
        for (k, v) in init_items {
            let boundary =
                roller.roll_bytes(&cjson::to_vec(&k).map_err(|err| format!("{:?}", err))?);
            current_block_items.push((k, v));
            if boundary {
                let header_key = current_block_items[0].0.clone();
                // TODO: serialize the block as a Map node or possibly node items.
                let block_bytes =
                    cjson::to_vec(&current_block_items).map_err(|err| format!("{:?}", err))?;
                current_block_items.clear();
                let block_hash = <[u8; 32]>::from(blake3::hash(&block_bytes));
                let block_addr = multibase::encode(Base::Base58Btc, &block_bytes);
                storage.write(&block_addr, &*block_bytes);
                headers.push((header_key, block_addr));
            }
        }
        // TODO: reduce this code duplication.
        if !current_block_items.is_empty() {
            let header_key = current_block_items[0].0.clone();
            // TODO: serialize the block as a Map node or possibly node items.
            let block_bytes =
                cjson::to_vec(&current_block_items).map_err(|err| format!("{:?}", err))?;
            current_block_items.clear();
            let block_hash = <[u8; 32]>::from(blake3::hash(&block_bytes));
            let block_addr = multibase::encode(Base::Base58Btc, &block_bytes);
            headers.push((header_key, block_addr));
        }
        Ok(Self { items: Vec::new() })
    }
    // fn boundary
    pub fn load<S>(storage: &S, map_ref: Ref) -> Self
    where
        S: StorageWrite,
    {
        todo!()
    }
}
/// An eventual abstraction over the real rolled hashing impl.
///
/// Right now though, it's using a fake rolled hashing. Assume very poor performance.
/// We'll add a buzhash eventually.
///
/// This is a really, really bad impl, on purpose. Need a proper roll.
pub struct RollHasher {
    window_size: usize,
    bytes: Vec<u8>,
}
impl RollHasher {
    pub fn new(window_size: usize) -> Self {
        log::warn!("a temporary non rolling hasher is being used, replace me!");
        Self {
            window_size,
            bytes: Vec::new(),
        }
    }
    pub fn roll_byte(&mut self, b: u8) -> bool {
        self.bytes.push(b);
        if self.bytes.len() > self.window_size {
            self.bytes.remove(0);
        }
        // super silly, but just making it compile before buzhash.
        let hash = <[u8; 32]>::from(blake3::hash(&self.bytes));
        let hash = u32::from_ne_bytes([hash[0], hash[1], hash[2], hash[3]]);
        hash & CHUNK_PATTERN == CHUNK_PATTERN
    }
    pub fn roll_bytes(&mut self, bytes: &[u8]) -> bool {
        for &b in bytes {
            if !self.roll_byte(b) {
                return false;
            }
        }
        true
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
        // let mut p = Prolly::new();
        Map::new(
            &storage,
            hashmap! {
                1 => 10,
                2 => 20,
            },
        );
        dbg!(&storage);
        let data = (0..20).map(|i| (i, i * 10)).collect::<HashMap<_, _>>();
        let m = Map::new(&storage, data);
        dbg!(&storage);
    }
    #[test]
    fn equality() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        let datas = vec![
            hashmap! {
                1 => 10,
                2 => 20,
            },
            (0..20).map(|i| (i, i * 10)).collect::<HashMap<_, _>>(),
        ];
        for data in datas {
            let storage_a = Memory::new();
            Map::new(&storage_a, data.clone()).unwrap();
            let storage_b = Memory::new();
            Map::new(&storage_b, data).unwrap();
            assert_eq!(storage_a, storage_b);
        }
    }
}
