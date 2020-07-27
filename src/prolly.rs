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
    Bool,
    Int,
    String,
    Blob,
    Ref { key: Vec<u8>, addr: Vec<u8> },
}
pub enum Ref {
    Blob(Vec<u8>),
}
pub struct Prolly {}
impl Prolly {
    pub fn new() -> Self {
        Self {}
    }
    pub fn flush<S>(&mut self, storage: S) -> Result<(), ()>
    where
        S: Storage,
    {
        todo!()
    }
    pub fn list(&mut self) -> List {
        todo!()
    }
}
struct List {}
impl List {
    pub fn append(&mut self, value: Value) {
        todo!()
    }
}
