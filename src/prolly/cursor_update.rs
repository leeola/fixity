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
    std::{collections::HashMap, mem},
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
    /// Roll into `target_k` but **do not** roll the KV pair equal to `target_k`; instead
    /// dropping that equal pair.
    pub async fn roll_into(&mut self, target_k: &Key) -> Result<(), Error> {
        // if there are no values in source_kvs and rolled_kvs then we need to attempt to load the
        // leaf block for the provided key.
        if self.source_kvs.is_empty() && self.rolled_kvs.is_empty() {
            if let Some(mut leaf) = self.reader.within_leaf_owned(target_k).await? {
                leaf.reverse();
                self.source_kvs.append(&mut leaf);
            }
        }
        // the resulting source_kvs may still be empty, or may contain only a single k:v
        // which matches the `target_k`. This is supported / expected.

        // now we roll the source_kvs up, one by one, so that this cursor is at the target.
        loop {
            // Peek at the upcoming cursor_key to see if it would be past the target_k.
            // If it is, we don't want to pop it - we've rolled into the target successfully.
            let kv = match self.source_kvs.last() {
                Some((cursor_k, _)) if cursor_k > target_k => {
                    return Ok(());
                }
                Some((cursor_k, _)) if cursor_k == target_k => {
                    self.source_kvs.pop();
                    return Ok(());
                }
                Some(_) => self.source_kvs.pop().expect("last kv impossibly missing"),
                None => return Ok(()),
            };
            self.roll_kv(kv).await?;
        }
    }
    pub async fn roll_kv(&mut self, kv: (Key, Value)) -> Result<(), Error> {
        let boundary = self.roller.roll_bytes(&crate::value::serialize(&kv)?);
        self.rolled_kvs.push(kv);
        if boundary {
            if self.rolled_kvs.len() == 1 {
                log::warn!(
                    "writing key & value that exceeds block size, this is highly inefficient"
                );
            }
            let (node_key, node_addr) = {
                let kvs = mem::replace(&mut self.rolled_kvs, Vec::new());
                let node = Node::<_, Value, _>::Leaf(kvs);
                let (node_addr, node_bytes) = node.as_bytes()?;
                self.storage.write(node_addr.clone(), &*node_bytes).await?;
                (node.into_key_unchecked(), node_addr)
            };
            let storage = &self.storage;
            let roller_config = &self.roller_config;
            // self.parent
            //     .get_or_insert_with(|| Box::new(Branch::new(storage, roller_config.clone())))
            //     .push((node_key, node_addr.into()))
            //     .await?;
        }
        Ok(())
    }
    pub async fn insert(&mut self, k: Key, v: Value) -> Result<(), Error> {
        self.roll_into(&k).await?;
        todo!("insert")
    }
    pub async fn remove(&mut self, k: Key) -> Result<(), Error> {
        self.roll_into(&k).await?;
        todo!("leaf remove")
    }
}
enum Resp {
    Addr(Addr),
}
struct Branch<'s, S> {
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
    parent: Option<Box<Branch<'s, S>>>,
}
impl<'s, S> Branch<'s, S> {
    pub fn new(storage: &'s S, root_addr: Addr, roller_config: RollerConfig) -> Self {
        Self {
            storage,
            reader: CursorRead::new(storage, root_addr),
            roller_config,
            roller: Roller::with_config(roller_config.clone()),
            rolled_kvs: Vec::new(),
            source_kvs: Vec::new(),
            parent: None,
        }
    }
}
impl<'s, S> Branch<'s, S>
where
    S: StorageRead + StorageWrite,
{
    pub async fn flush(&mut self) -> Result<Addr, Error> {
        todo!()
    }
}
