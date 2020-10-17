// TODO: remove allow(unused)
#![allow(unused)]

use {
    crate::{
        prolly::{
            node::Node,
            roller::{Config as RollerConfig, Roller},
        },
        storage::{StorageRead, StorageWrite},
        value::{Addr, Key, Value},
        Error,
    },
    std::collections::HashMap,
};

pub struct CursorUpdate<'s, S> {
    leaf: Leaf<'s, S>,
}
impl<'s, S> CursorUpdate<'s, S> {
    pub fn new(storage: &'s S) -> Self {
        Self::with_roller(storage, RollerConfig::default())
    }
    pub fn with_roller(storage: &'s S, roller_config: RollerConfig) -> Self {
        Self {
            leaf: Leaf::new(storage, roller_config),
        }
    }
}
impl<'s, S> CursorUpdate<'s, S>
where
    S: StorageRead + StorageWrite,
{
    pub async fn with_hashmap(mut self, kvs: HashMap<Key, Value>) -> Result<Addr, Error> {
        let mut kvs = kvs.into_iter().collect::<Vec<_>>();
        // unstable should be fine, since the incoming values are unique.
        kvs.sort_unstable();
        for kv in kvs.into_iter() {
            self.leaf.push(kv).await?;
        }
        self.leaf.flush().await
    }
}
struct Leaf<'s, S> {
    storage: &'s S,
    roller_config: RollerConfig,
    roller: Roller,
    buffer: Vec<(Key, Value)>,
    // parent: Option<Branch<'s, S>>,
}
impl<'s, S> Leaf<'s, S> {
    pub fn new(storage: &'s S, roller_config: RollerConfig) -> Self {
        Self {
            storage,
            roller_config,
            roller: Roller::with_config(roller_config.clone()),
            buffer: Vec::new(),
            // parent: None,
        }
    }
}
impl<'s, S> Leaf<'s, S>
where
    S: StorageRead + StorageWrite,
{
    pub async fn flush(&mut self) -> Result<Addr, Error> {
        todo!()
    }
    pub async fn push(&mut self, kv: (Key, Value)) -> Result<(), Error> {
        todo!()
    }
}
