#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use {
    crate::{
        prolly::roller::{Config as RollerConfig, Roller},
        storage::{Storage, StorageRead, StorageWrite},
        Addr, Error,
    },
    std::{cmp::Eq, collections::HashMap, hash::Hash},
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Update<V> {
    Insert(V),
    Remove,
}
pub struct Tree<K, V> {
    updates: HashMap<K, Update<V>>,
}
impl<K, V> Tree<K, V>
where
    // S: StorageWrite,
    K: std::fmt::Debug + Eq + Hash,
{
    pub fn new() -> Self {
        todo!()
    }
    pub fn insert(&mut self, k: K, v: V) {
        self.updates.insert(k, Update::Insert(v));
    }
    pub fn remove(&mut self, k: K) {
        self.updates.insert(k, Update::Remove);
    }
}
impl<K, V> Tree<K, V>
where
    // S: StorageWrite,
    K: std::fmt::Debug + Serialize + Clone,
    V: std::fmt::Debug + Serialize,
{
    pub fn commit(&mut self) {
        todo!()
    }
}
