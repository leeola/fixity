use {
    crate::{
        prolly::{
            roller::{Config as RollerConfig, Roller},
            CursorRead, NodeOwned,
        },
        storage::{StorageRead, StorageWrite},
        value::{Addr, Key, Value},
        Error,
    },
    std::{collections::HashMap, mem},
};

pub struct CursorUpdate<'s, S> {
    /// The last used key, if any, to ensure forward progress on the cursor behavior if
    /// debug checks are enabled.
    #[cfg(features = "debug_checks")]
    cursor_key: Option<Key>,
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
        #[cfg(features = "debug_checks")]
        if let Some(cursor_key) = self.cursor_key.replace(k.clone()) {
            if cursor_key >= k {
                panic!(
                    "cursor update did not move forward. from key `{}` to `{}`",
                    cursor_key, k
                );
            }
        }
        self.leaf.insert(k, v).await
    }
    pub async fn remove(&mut self, k: Key) -> Result<(), Error> {
        #[cfg(features = "debug_checks")]
        if let Some(cursor_key) = self.cursor_key.replace(k.clone()) {
            if cursor_key >= k {
                panic!(
                    "cursor update did not move forward. from key `{}` to `{}`",
                    cursor_key, k
                );
            }
        }
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
    root_addr: Addr,
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
    source_depth: usize,
    parent: Option<Branch<'s, S>>,
}
impl<'s, S> Leaf<'s, S> {
    pub fn new(storage: &'s S, root_addr: Addr, roller_config: RollerConfig) -> Self {
        Self {
            storage,
            reader: CursorRead::new(storage, root_addr.clone()),
            root_addr,
            roller_config,
            roller: Roller::with_config(roller_config.clone()),
            // NIT: we're wasting some initial allocation here. iirc this was done because
            // Async/Await doesn't like constructors - eg `Self` returns.
            rolled_kvs: Vec::new(),
            source_kvs: Vec::new(),
            source_depth: 0,
            parent: None,
        }
    }
}
impl<'s, S> Leaf<'s, S>
where
    S: StorageRead + StorageWrite,
{
    pub async fn flush(&mut self) -> Result<Addr, Error> {
        let source_kvs = mem::replace(&mut self.source_kvs, Vec::new());
        for kv in source_kvs.into_iter() {
            self.roll_kv(kv).await?;
        }
        if self.rolled_kvs.is_empty() {
            match self.parent.take() {
                // If there's no parent, this Leaf never hit a Boundary and thus this
                // Leaf itself is the root.
                //
                // This should be impossible.
                // A proper state machine would make this logic more safe, but async/await is
                // currently a bit immature for the design changes that would introduce.
                None => unreachable!("CursorUpdate leaf missing parent and has empty buffer"),
                // If there is a parent, the root might be the parent, grandparent, etc.
                Some(mut parent) => parent.flush(None).await,
            }
        } else {
            let (node_key, node_addr) = {
                let kvs = mem::replace(&mut self.rolled_kvs, Vec::new());
                let node = NodeOwned::Leaf(kvs);
                let (node_addr, node_bytes) = node.as_bytes()?;
                self.storage.write(node_addr.clone(), &*node_bytes).await?;
                (node.into_key_unchecked(), node_addr)
            };
            match self.parent.take() {
                // If there's no parent, this Leaf never hit a Boundary and thus this
                // instance itself is the root.
                None => Ok(node_addr),
                // If there is a parent, the root will be the parent, or grandparent, etc.
                Some(mut parent) => parent.flush(Some((node_key, node_addr))).await,
            }
        }
    }
    /// Roll into `target_k` but **do not** roll the KV pair equal to `target_k`; instead
    /// dropping that equal pair.
    pub async fn roll_into(&mut self, target_k: &Key) -> Result<(), Error> {
        // roll the source_kvs up, one by one, so that this cursor is at the target.
        loop {
            match self.source_kvs.last() {
                // If the cursor is past the target, return - we can insert freely.
                Some((cursor_k, _)) if cursor_k > target_k => {
                    return Ok(());
                }
                // If the cursor is at the target, remove it and return.
                // both Self::insert() and Self::remove() result in the old matching value
                // getting removed.
                Some((cursor_k, _)) if cursor_k == target_k => {
                    self.source_kvs.pop();
                    return Ok(());
                }
                // If the cursor is before the target, roll the kv.
                Some(_) => {
                    let kv = self.source_kvs.pop().expect("last kv impossibly missing");
                    self.roll_kv(kv).await?;
                }
                // If the is no more source_kvs, load more data. Either the window for
                // the target_key, or we expand the existing window.
                None => {
                    // if there is no more source kvs, we either need to expand the window or
                    // load an entirely new window to complete the rolled_into request.
                    self.expand_window(target_k).await?;
                    // if we tried to expand the window but failed, there is no more matching
                    // data for this window. If rolled_kvs is not empty this is fine,
                    // we can just append the target_k.
                    if self.source_kvs.is_empty() {
                        // If we don't return, this loop is infinite.
                        return Ok(());
                    }
                }
            };
        }
    }
    pub async fn expand_window(&mut self, target_k: &Key) -> Result<(), Error> {
        // if there is no more source kvs, we either need to expand the window or
        // load an entirely new window to complete the rolled_into request.
        if self.rolled_kvs.is_empty() {
            if let Some(mut leaf) = self.reader.leaf_matching_key_owned(target_k).await? {
                if let Some(parent) = self.parent.as_mut() {
                    if let Some((k, _)) = leaf.inner.get(0) {
                        // inform our parent that this leaf is (might be) mutating, causing
                        // the parent to remove it from the list of `(Key,Addr)` pairs.
                        //
                        // The key:addr pair gets added back when we roll into a new boundary,
                        // which can change due to mutation.
                        parent.mutating_child_key(k).await?;
                    }
                }

                leaf.inner.reverse();
                self.source_kvs.append(&mut leaf.inner);
                // NIT: this feels.. bad. I debated an init phase where after the constructor we
                // figure the depth of this block, so we only do it once - but then we're traversing the
                // tree at construction just to find this pointer. Where as the depth
                // is basically free anytime we get a block. So... fixing this seems like a super
                // micro-optimization
                self.source_depth = leaf.depth;
            }
        } else {
            let neighbor_to = self.rolled_kvs.last().expect("impossibly missing");
            if let Some(mut leaf) = self.reader.leaf_right_of_key_owned(&neighbor_to.0).await? {
                leaf.reverse();
                self.source_kvs.append(&mut leaf);
            }
        }
        Ok(())
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
                let node = NodeOwned::Leaf(kvs);
                let (node_addr, node_bytes) = node.as_bytes()?;
                self.storage.write(node_addr.clone(), &*node_bytes).await?;
                (node.into_key_unchecked(), node_addr)
            };
            let storage = &self.storage;
            let roller_config = &self.roller_config;
            let root_addr = self.root_addr.clone();
            let branch_depth = self.source_depth - 1;
            self.parent
                .get_or_insert_with(|| {
                    Branch::new(storage, root_addr, roller_config.clone(), branch_depth)
                })
                .push((node_key, node_addr.into()))
                .await?;
        }
        Ok(())
    }
    pub async fn insert(&mut self, k: Key, v: Value) -> Result<(), Error> {
        self.roll_into(&k).await?;
        self.roll_kv((k, v)).await?;
        Ok(())
    }
    pub async fn remove(&mut self, k: Key) -> Result<(), Error> {
        self.roll_into(&k).await?;
        Ok(())
    }
}
struct Branch<'s, S> {
    storage: &'s S,
    reader: CursorRead<'s, S>,
    root_addr: Addr,
    roller_config: RollerConfig,
    roller: Roller,
    /// Rolled KVs in sorted order, to be eventually written to Storage once a boundary
    /// is found via the Roller.
    rolled_kvs: Vec<(Key, Addr)>,
    /// KVs being merged in one by one, as the cursor progresses via `insert()` and
    /// `remove()` methods.
    ///
    /// These are stored in **reverse order**, allowing removal of values at low cost.
    source_kvs: Vec<(Key, Addr)>,
    source_depth: usize,
    parent: Option<Box<Branch<'s, S>>>,
}
impl<'s, S> Branch<'s, S> {
    pub fn new(
        storage: &'s S,
        root_addr: Addr,
        roller_config: RollerConfig,
        source_depth: usize,
    ) -> Self {
        Self {
            storage,
            reader: CursorRead::new(storage, root_addr.clone()),
            root_addr,
            roller_config,
            roller: Roller::with_config(roller_config.clone()),
            // NIT: we're wasting some initial allocation here. iirc this was done because
            // Async/Await doesn't like constructors - eg `Self` returns.
            rolled_kvs: Vec::new(),
            source_kvs: Vec::new(),
            source_depth,
            parent: None,
        }
    }
}
impl<'s, S> Branch<'s, S>
where
    S: StorageRead + StorageWrite,
{
    #[async_recursion::async_recursion]
    pub async fn flush(&mut self, kv: Option<(Key, Addr)>) -> Result<Addr, Error> {
        if let Some(kv) = kv {
            self.push(kv).await?;
        }
        let source_kvs = mem::replace(&mut self.source_kvs, Vec::new());
        for kv in source_kvs.into_iter() {
            self.roll_kv(kv).await?;
        }
        if self.rolled_kvs.is_empty() {
            match self.parent.take() {
                // If there's no parent, this Leaf never hit a Boundary and thus this
                // Leaf itself is the root.
                //
                // This should be impossible.
                // A proper state machine would make this logic more safe, but async/await is
                // currently a bit immature for the design changes that would introduce.
                None => unreachable!("CursorUpdate leaf missing parent and has empty buffer"),
                // If there is a parent, the root might be the parent, grandparent, etc.
                Some(mut parent) => parent.flush(None).await,
            }
        } else {
            let (node_key, node_addr) = {
                let kvs = mem::replace(&mut self.rolled_kvs, Vec::new());
                let node = NodeOwned::Branch(kvs);
                let (node_addr, node_bytes) = node.as_bytes()?;
                self.storage.write(node_addr.clone(), &*node_bytes).await?;
                (node.into_key_unchecked(), node_addr)
            };
            match self.parent.take() {
                // If there's no parent, this Leaf never hit a Boundary and thus this
                // instance itself is the root.
                None => Ok(node_addr),
                // If there is a parent, the root will be the parent, or grandparent, etc.
                Some(mut parent) => parent.flush(Some((node_key, node_addr))).await,
            }
        }
    }
    /// Roll this `Branch`'s window into `target_k` but **do not** roll the KV pair equal to
    /// `target_k`; instead dropping that equal pair.
    #[async_recursion::async_recursion]
    pub async fn roll_into(&mut self, target_k: &Key) -> Result<(), Error> {
        // roll the source_kvs up, one by one, so that this cursor is at the target.
        loop {
            match self.source_kvs.last() {
                // If the cursor is past the target, return - we can insert freely.
                Some((cursor_k, _)) if cursor_k > target_k => {
                    return Ok(());
                }
                // If the cursor is at the target, remove it and return.
                // both Self::insert() and Self::remove() result in the old matching value
                // getting removed.
                Some((cursor_k, _)) if cursor_k == target_k => {
                    self.source_kvs.pop();
                    return Ok(());
                }
                // If the cursor is before the target, roll the kv.
                Some(_) => {
                    let kv = self.source_kvs.pop().expect("last kv impossibly missing");
                    self.roll_kv(kv).await?;
                }
                // If the is no more source_kvs, load more data. Either the window for
                // the target_key, or we expand the existing window.
                None => {
                    // if there is no more source kvs, we either need to expand the window or
                    // load an entirely new window to complete the rolled_into request.
                    self.expand_window(target_k).await?;
                    // if we tried to expand the window but failed, there is no more matching
                    // data for this window. If rolled_kvs is not empty this is fine,
                    // we can just append the target_k.
                    if self.source_kvs.is_empty() {
                        // If we don't return, this loop is infinite.
                        return Ok(());
                    }
                }
            };
        }
    }
    /// Expand the `Branch` `Key:Addr` window.
    #[async_recursion::async_recursion]
    pub async fn expand_window(&mut self, target_k: &Key) -> Result<(), Error> {
        // if there is no more source kvs, we either need to expand the window or
        // load an entirely new window to complete the rolled_into request.

        // If the rolled_kvs vec is empty, the current window is "clean" - aka no modifications
        // that need to be cleaned up by expanding the window. So instead of expanding, we load
        // the correct window for `target_k`.
        if self.rolled_kvs.is_empty() {
            if let Some(mut branch) = self
                .reader
                .branch_matching_key_owned(target_k, self.source_depth)
                .await?
            {
                if let Some(parent) = self.parent.as_mut() {
                    if let Some((k, _)) = branch.get(0) {
                        // inform our parent that this branch is (might be) mutating, causing
                        // the parent to remove it from the list of `(Key,Addr)` pairs.
                        //
                        // The key:addr pair gets added back when we roll into a new boundary,
                        // which can change due to mutation.
                        parent.mutating_child_key(k).await?;
                    }
                }

                branch.reverse();
                self.source_kvs.append(&mut branch);
            }
        // If the rolled_kvs vec is not empty we need to expand the source_kvs to either find
        // a border, or roll into the `target_k`.
        } else {
            let neighbor_to = self.rolled_kvs.last().expect("impossibly missing");
            if let Some(mut branch) = self
                .reader
                .branch_right_of_key_owned(&neighbor_to.0, self.source_depth)
                .await?
            {
                branch.reverse();
                self.source_kvs.append(&mut branch);
            }
        }
        Ok(())
    }
    #[async_recursion::async_recursion]
    pub async fn roll_kv(&mut self, kv: (Key, Addr)) -> Result<(), Error> {
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
                let node = NodeOwned::Branch(kvs);
                let (node_addr, node_bytes) = node.as_bytes()?;
                self.storage.write(node_addr.clone(), &*node_bytes).await?;
                (node.into_key_unchecked(), node_addr)
            };
            let storage = &self.storage;
            let roller_config = &self.roller_config;
            let root_addr = self.root_addr.clone();
            let branch_depth = self.source_depth - 1;
            self.parent
                .get_or_insert_with(|| {
                    Box::new(Branch::new(
                        storage,
                        root_addr,
                        roller_config.clone(),
                        branch_depth,
                    ))
                })
                .push((node_key, node_addr.into()))
                .await?;
        }
        Ok(())
    }
    #[async_recursion::async_recursion]
    pub async fn mutating_child_key(&mut self, k: &Key) -> Result<(), Error> {
        // roll into pops the key, which is all branches ever care about doing.
        // So roll-into handles most of the logic.
        self.roll_into(k).await?;
        Ok(())
    }
    #[async_recursion::async_recursion]
    pub async fn push(&mut self, kv: (Key, Addr)) -> Result<(), Error> {
        self.roll_into(&kv.0).await?;
        self.roll_kv(kv).await?;
        Ok(())
    }
}
