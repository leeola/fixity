// pub mod create;
pub mod node;
// pub mod read;
pub mod roller;
// pub mod update;
pub use node::Node;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use {
    crate::{
        storage::{Storage, StorageRead, StorageWrite},
        Addr, Error,
    },
    std::collections::HashMap,
};
pub struct Ref {
    ref_type: RefType,
    addr: Addr,
}
pub enum RefType {
    Map,
}
pub mod types {
    pub struct Usize(usize);
}
pub enum ValueType {
    Usize,
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
    Uint32(u32),
    // String,
    // Blob,
    // Map(Map<Key, Box<Value>>),
    // Ref { key: Vec<u8>, addr: Vec<u8> },
}
impl From<u32> for Value {
    fn from(t: u32) -> Self {
        Self::Uint32(t)
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
pub struct Map<K, V> {
    // items: Vec<NodeItem<K, V>>,
    _pd: std::marker::PhantomData<(K, V)>,
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
        todo!("new map")
    }
    pub fn load<S>(storage: &S, map_ref: Ref) -> Self
    where
        S: StorageWrite,
    {
        todo!("map load")
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
        )
        .unwrap();
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
