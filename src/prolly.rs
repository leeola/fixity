use {
    crate::{
        storage::{Storage, StorageRead, StorageWrite},
        Addr,
    },
    std::collections::HashMap,
};
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
pub struct Ref<T>(T);
pub enum NodeItem<K, V, T> {
    Refs(Vec<(K, Ref<T>)>),
    Values(Vec<(K, V)>),
}
pub struct Map<K, V> {
    items: Vec<NodeItem<K, V, Map<K, V>>>,
}
impl<K, V> Map<K, V>
where
    K: Ord,
{
    // TODO: make the map generic.
    pub fn new<S>(storage: &S, map: HashMap<K, V>) -> Result<Self, String>
    where
        S: StorageWrite,
    {
        let mut init_items = map.into_iter().map(|(k, v)| (k, v)).collect::<Vec<_>>();
        init_items.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
        let blocks = Vec::new();

        Ok(Self { items: Vec::new() })
    }
    pub fn load<S>(storage: &S, map_ref: Ref<Map<K, V>>) -> Self
    where
        S: StorageWrite,
    {
        todo!()
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
        let mut m = Map::new(
            &storage,
            hashmap! {
                1 => 10,
            },
        );
    }
}
