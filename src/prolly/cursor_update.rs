// TODO: remove allow(unused)
#![allow(unused)]

use {
    crate::{
        prolly::{
            roller::{Config as RollerConfig, Roller},
            CursorRead, Node, NodeOwned,
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
    pub fn new(storage: &'s S, root_addr: Addr) -> Self {
        Self::with_roller(storage, root_addr, RollerConfig::default())
    }
    pub fn with_roller(storage: &'s S, root_addr: Addr, roller_config: RollerConfig) -> Self {
        Self {
            leaf: Leaf::new(storage, root_addr, roller_config),
        }
    }
}
impl<'s, S> CursorUpdate<'s, S>
where
    S: StorageRead + StorageWrite,
{
    pub async fn with_hashmap_changes(mut self, kchs: HashMap<Key, Change>) -> Result<Addr, Error> {
        let mut kchs = kchs.into_iter().collect::<Vec<_>>();
        // unstable should be fine, since the incoming values are unique.
        kchs.sort_unstable();
        for (k, ch) in kchs {
            self.change(k, ch).await?;
        }
        self.leaf.flush().await
    }
    pub async fn flush(&mut self) -> Result<Addr, Error> {
        self.leaf.flush().await
    }
    pub async fn insert(&mut self, k: Key, v: Value) -> Result<(), Error> {
        self.leaf.insert(k, v).await
    }
    pub async fn remove(&mut self, k: Key) -> Result<(), Error> {
        self.leaf.remove(k).await
    }
    pub async fn change(&mut self, k: Key, change: Change) -> Result<(), Error> {
        match change {
            Change::Insert(v) => self.insert(k, v).await,
            Change::Remove => self.remove(k).await,
        }
    }
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Change {
    Insert(Value),
    Remove,
}
struct Leaf<'s, S> {
    storage: &'s S,
    reader: CursorRead<'s, S>,
    roller_config: RollerConfig,
    roller: Roller,
    /// Rolled KVs in sorted order, to be eventually written to Storage once a boundary
    /// is found via the Roller.
    rolled_kvs: Vec<(Key, Value)>,
    /// KVs being merged in one by one, as the cursor progresses via `insert()` and
    /// `remove()` methods.
    ///
    /// These are stored in **reverse order**, allowing removal of values at low cost.
    source_kvs: Vec<(Key, Value)>,
    // parent: Option<Branch<'s, S>>,
}
impl<'s, S> Leaf<'s, S> {
    pub fn new(storage: &'s S, root_addr: Addr, roller_config: RollerConfig) -> Self {
        Self {
            storage,
            reader: CursorRead::new(storage, root_addr),
            roller_config,
            roller: Roller::with_config(roller_config.clone()),
            rolled_kvs: Vec::new(),
            source_kvs: Vec::new(),
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
    pub async fn roll_up_to(&mut self, k: &Key) -> Result<(), Error> {
        if self.source_kvs.is_empty() && self.rolled_kvs.is_empty() {
            if let Some(mut leaf) = self.reader.within_leaf_owned(k.clone()).await? {
                self.source_kvs.append(&mut leaf);
            }
        }
        loop {
            if self.source_kvs.is_empty() && !self.rolled_kvs.is_empty() {}
        }
        todo!()
    }
    pub async fn clean_up_to(&mut self, k: &Key) -> Result<(), Error> {
        todo!("clean_up_to")
    }
    pub async fn insert(&mut self, k: Key, v: Value) -> Result<(), Error> {
        todo!("insert")
    }
    pub async fn remove(&mut self, k: Key) -> Result<(), Error> {
        todo!("leaf remove")
    }
}
