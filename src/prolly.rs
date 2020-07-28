use crate::storage::{Storage, StorageRead, StorageWrite};
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
    pub fn new_list(&mut self) -> List {
        todo!()
    }
}
pub struct List {}
impl List {
    pub fn new() -> Self {
        Self {}
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
        let s = Memory::new();
        // let mut p = Prolly::new();
        let mut l = List::new();
        l.append(1);
        dbg!(l.commit(&s).unwrap());
    }
}
