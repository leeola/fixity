use {
    crate::storage::{Storage, StorageRead, StorageWrite},
    std::collections::HashMap,
};
pub enum Node {
    NodeRefs(Vec<NodeRef>),
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
    Bool,
    Int,
    String,
    Blob,
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
pub enum Ref {
    // Blob(Vec<u8>),
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
    pub fn new_list(&mut self) -> List<T> {
        todo!()
    }
}
pub struct List<T> {
    len: usize,
    // inserted: HashMap<usize, T>,
    appended: Vec<T>,
}
impl<T> List<T> {
    pub fn new() -> Self {
        Self {
            len: usize,
            appended: Vec::new(),
        }
    }
    pub fn commit<S>(&mut self, storage: &S) -> Result<Ref, String>
    where
        S: Storage,
    {
        todo!()
    }
    pub fn append<T>(&mut self, value: T)
    where
        T: Into<Value>,
    {
        todo!()
    }
}
enum MapChange {
    Insert((Key, Value)),
    Remove(Key),
}
pub struct Map {
    len: usize,
    staged: Vec<MapChange>,
}
impl<K, V> Map<K, V> {
    pub fn new() -> Self {
        Self {
            len: usize,
            staged: Vec::new(),
        }
    }
    pub fn commit<S>(&mut self, storage: &S) -> Result<Ref, String>
    where
        S: Storage,
    {
        todo!()
    }
    pub fn insert<T, U>(&mut self, k: T, v: U)
    where
        T: Into<Key>,
        U: Into<Value>,
    {
        todo!()
    }
}

#[cfg(test)]
pub mod test {
    use {
        super::*,
        crate::storage::{Memory, Storage, StorageRead, StorageWrite},
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
        let mut m = Map::new();
        m.insert(1, 10);
        dbg!(m.commit(&storage).unwrap());
    }
}
