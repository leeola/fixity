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

pub struct NaiveProlly {}

impl NaiveProlly {
    pub fn new() -> Self {
        Self {}
    }
    fn append(key: &[u8], value: &[u8]) -> bool {
        todo!()
    }
}
