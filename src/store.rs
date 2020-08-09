use {
    crate::{Addr, Commit, Id, Result},
    async_trait::async_trait,
    std::io::{Read, Write},
};
#[cfg(feature = "serde")]
#[async_trait]
pub trait Store {
    // fn get<T>(&self, r: Ref<T>) -> Result<T, ()>
    // where
    //     T: serde::Deserialize;
    // fn put<T>(&self, r: Ref<T>) -> Result<T, ()>
    // where
    //     T: serde::Serialize;
    // fn put_read(&self, bytes: &mut dyn Read) -> Result<Addr>;
    // fn new() -> Id;
    // fn push(content: T, id: Option<Id>) -> Result<Commit>;
    // fn clone() -> ();
}

// fn map<K,V>(&self, r: Ref<Map<K,V>>) -> Result<Map<K,V>, ()>;
// fn get<T>(&self, r: Ref<T>) -> Result<T, ()>;
