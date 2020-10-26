use {
    crate::{
        prolly::{
            cursor_read::Block,
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
        dbg!(&root_addr);
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
        while let Some(kv) = self.source_kvs.pop() {
            self.roll_kv(kv).await?;
        }
        dbg!(&self.rolled_kvs);
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
                self.notify_parent_of_mutation(&leaf).await?;
                leaf.inner.reverse();
                leaf.inner.append(&mut self.source_kvs);
                self.source_kvs = leaf.inner;
                // NIT: this feels.. bad. I debated an init phase where after the constructor we
                // figure the depth of this block, so we only do it once - but then we're traversing the
                // tree at construction just to find this pointer. Where as the depth
                // is basically free anytime we get a block. So... fixing this seems like a super
                // micro-optimization
                self.source_depth = dbg!(leaf.depth);
            }
        } else {
            let neighbor_to = self.rolled_kvs.last().expect("impossibly missing");
            if let Some(mut leaf) = self.reader.leaf_right_of_key_owned(&neighbor_to.0).await? {
                self.notify_parent_of_mutation(&leaf).await?;
                leaf.inner.reverse();
                leaf.inner.append(&mut self.source_kvs);
                self.source_kvs = leaf.inner;
            }
        }
        Ok(())
    }
    pub async fn notify_parent_of_mutation(&mut self, block: &Block<Value>) -> Result<(), Error> {
        // conceptually if the source data does not expand to the parent, there's no need
        // to notify the parent about mutations of the source data.
        if block.depth == 0 {
            return Ok(());
        }
        let parent = {
            let storage = &self.storage;
            let roller_config = &self.roller_config;
            let reader = &self.reader;
            self.parent.get_or_insert_with(|| {
                let branch_reader = BranchReader::from_leaf(block.depth, &reader);
                Branch::new(storage, roller_config.clone(), branch_reader)
            })
        };
        if let Some((k, _)) = block.inner.get(0usize) {
            // inform our parent that this leaf is (might be) mutating, causing
            // the parent to remove it from the list of `(Key,Addr)` pairs.
            //
            // The key:addr pair gets added back when we roll into a new boundary,
            // which can change due to mutation.
            parent.mutating_child_key(k).await?;
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
            let source_depth = &self.source_depth;
            let reader = &self.reader;
            self.parent
                .get_or_insert_with(|| {
                    let branch_reader = BranchReader::from_leaf(*source_depth, &reader);
                    Branch::new(storage, roller_config.clone(), branch_reader)
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
/// A `Branch` helper to store the source depth and reader in the same struct.
struct BranchReader<'s, S> {
    /// The depth in the source tree, not actively of the currently updating branch.
    pub depth: usize,
    /// A reader for the source tree.
    pub reader: CursorRead<'s, S>,
}
impl<'s, S> BranchReader<'s, S> {
    /// Create a BranchReader from the leaf, subtracting the depth as needed.
    pub fn from_leaf(depth: usize, leaf_reader: &CursorRead<'s, S>) -> Option<Self> {
        if depth > 0 {
            Some(BranchReader {
                depth: depth - 1,
                reader: leaf_reader.clone(),
            })
        } else {
            None
        }
    }
    /// Return a `BranchReader` for the parent branch, if any.
    ///
    /// # Returns
    ///
    /// If this branch is already at source root, this returns None;
    /// otherwise it returns a reader and depth for the parent.
    pub fn parent(&self) -> Option<Self> {
        if self.depth > 0 {
            Some(Self {
                depth: self.depth - 1,
                reader: self.reader.clone(),
            })
        } else {
            None
        }
    }
}
struct Branch<'s, S> {
    storage: &'s S,
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
    /// A reader and depth for the source material that this branch depth is updating.
    ///
    /// Since this is assigned on creation, if None this Branch has no source material,
    /// meaning that the child has expanded the tree depth such that the source
    /// tree no longer overlaps.
    source: Option<BranchReader<'s, S>>,
    parent: Option<Box<Branch<'s, S>>>,
}
impl<'s, S> Branch<'s, S> {
    pub fn new(
        storage: &'s S,
        roller_config: RollerConfig,
        source: Option<BranchReader<'s, S>>,
    ) -> Self {
        Self {
            storage,
            roller_config,
            roller: Roller::with_config(roller_config.clone()),
            // NIT: we're wasting some initial allocation here. iirc this was done because
            // Async/Await doesn't like constructors - eg `Self` returns.
            rolled_kvs: Vec::new(),
            source_kvs: Vec::new(),
            source,
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
        dbg!(&kv, &self.rolled_kvs, &self.source_kvs);
        // push will roll into kv
        if let Some(kv) = kv {
            self.push(kv).await?;
        }
        while let Some(kv) = self.source_kvs.pop() {
            self.roll_kv(kv).await?;
        }
        if self.rolled_kvs.is_empty() {
            match self.parent.take() {
                // If there's no parent, this Branch never hit a Boundary and thus this
                // Branch itself is the root.
                //
                // This should be impossible.
                // A proper state machine would make this logic more safe, but async/await is
                // currently a bit immature for the design changes that would introduce.
                None => unreachable!("CursorUpdate branch missing parent and has empty buffer"),
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
                // If there's no parent, this Branch never hit a Boundary and thus this
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

        // but only if we have a source. If we don't, there's no expand possible.
        if let Some(source) = self.source.as_mut() {
            // If the rolled_kvs vec is empty, the current window is "clean" - aka no modifications
            // that need to be cleaned up by expanding the window. So instead of expanding, we load
            // the correct window for `target_k`.
            if self.rolled_kvs.is_empty() {
                if let Some(mut branch) = source
                    .reader
                    .branch_matching_key_owned(target_k, source.depth)
                    .await?
                {
                    self.notify_parent_of_mutation(&branch).await?;
                    branch.reverse();
                    branch.append(&mut self.source_kvs);
                    self.source_kvs = branch;
                }
            // If the rolled_kvs vec is not empty we need to expand the source_kvs to either find
            // a border, or roll into the `target_k`.
            } else {
                let neighbor_to = self.rolled_kvs.last().expect("impossibly missing");
                if let Some(mut branch) = source
                    .reader
                    .branch_right_of_key_owned(&neighbor_to.0, source.depth)
                    .await?
                {
                    self.notify_parent_of_mutation(&branch).await?;
                    branch.reverse();
                    branch.append(&mut self.source_kvs);
                    self.source_kvs = branch;
                }
            }
        }
        Ok(())
    }
    pub async fn notify_parent_of_mutation(&mut self, block: &[(Key, Addr)]) -> Result<(), Error> {
        // conceptually if the source data does not expand to the parent, there's no need
        // to notify the parent about mutations of the source data.
        let source = match &self.source {
            Some(source) if source.depth == 0 => return Ok(()),
            None => return Ok(()),
            Some(source) => source,
        };
        let parent = {
            let storage = &self.storage;
            let roller_config = &self.roller_config;
            self.parent.get_or_insert_with(|| {
                Box::new(Branch::new(storage, roller_config.clone(), source.parent()))
            })
        };
        if let Some((k, _)) = block.get(0usize) {
            // inform our parent that this leaf is (might be) mutating, causing
            // the parent to remove it from the list of `(Key,Addr)` pairs.
            //
            // The key:addr pair gets added back when we roll into a new boundary,
            // which can change due to mutation.
            parent.mutating_child_key(k).await?;
        }
        Ok(())
    }
    #[async_recursion::async_recursion]
    pub async fn roll_kv(&mut self, kv: (Key, Addr)) -> Result<(), Error> {
        dbg!(&kv);
        let boundary = self.roller.roll_bytes(&crate::value::serialize(&kv)?);
        self.rolled_kvs.push(kv);
        if boundary {
            if self.rolled_kvs.len() == 1 {
                log::warn!(
                    "writing key & value that exceeds block size, this is highly inefficient"
                );
                // In the event that the KV is a boundary itself, we run into the risk of the
                // single block being pushed to a parent, which then also encounters a single block.
                // This is unlikely with a proper average block size, but still a design flaw
                // in this implementation. The next CursorUpdate implementation needs to resolve this.
                if self.source.is_none() && self.parent.is_none() {
                    log::debug!("not pushing new parent on single length branch block");
                    return Ok(());
                }
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
            dbg!("sending to parent..");
            self.parent
                .get_or_insert_with(|| {
                    Box::new(Branch::new(
                        storage,
                        roller_config.clone(),
                        // notify_parent_of_mutation will always expand parents for all source depths.
                        // if we ever expand parents here, we have to be past all possible sources,
                        // and thus this value can always be None.
                        None,
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
#[cfg(test)]
pub mod test {
    use {
        super::*,
        crate::prolly::{debug_read::DebugNode, roller::Config as RollerConfig, CursorCreate},
        crate::storage::Memory,
    };
    /// A smaller value to use with the roller, producing smaller average block sizes.
    const TEST_PATTERN: u32 = (1 << 8) - 1;
    #[tokio::test]
    async fn addr_after_mutations() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        let test_cases = vec![
            (
                "left of single node tree",
                (0..3),
                vec![
                    vec![(0.into(), Change::Remove)],
                    vec![(0.into(), Change::Insert(0.into()))],
                ],
            ),
            (
                "mutating with a branch",
                (0..7),
                vec![
                    vec![(0.into(), Change::Remove), (2.into(), Change::Remove)],
                    vec![
                        (0.into(), Change::Insert(0.into())),
                        (2.into(), Change::Insert(2.into())),
                    ],
                ],
            ),
        ];
        for (test_desc, kvs, change_sets) in test_cases.into_iter().nth(1) {
            log::info!("test: {}", test_desc);
            log::info!("test: {}", test_desc);
            log::info!("test: {}", test_desc);
            let kvs = kvs
                .map(|i| (i, i))
                .map(|(k, v)| (Key::from(k), Value::from(v)))
                .collect::<Vec<_>>();
            let storage = Memory::new();
            let original_addr = {
                let tree =
                    CursorCreate::with_roller(&storage, RollerConfig::with_pattern(TEST_PATTERN));
                tree.with_kvs(kvs).await.unwrap()
            };
            let mut change_set_addr = original_addr.clone();
            for change_set in change_sets {
                log::trace!("new update cursor {:?}", change_set_addr);
                let mut tree = CursorUpdate::with_roller(
                    &storage,
                    change_set_addr.clone(),
                    RollerConfig::with_pattern(TEST_PATTERN),
                );
                for (k, change) in change_set {
                    log::info!("key:{:?}, change:{:?}", k, change);
                    tree.change(k, change).await.unwrap();
                }
                change_set_addr = tree.flush().await.unwrap();
                log::info!("flushed into {:?}", change_set_addr);
                DebugNode::new(&storage, &change_set_addr)
                    .await
                    .unwrap()
                    .print();
            }
            assert_eq!(
                original_addr, change_set_addr,
                "mutating a tree and then restoring the content should result in the same Addr",
            )
        }
    }
    enum MutState {
        Inserted,
        Removed,
    }
    fn rand_mutations(
        seed: u64,
        limit: usize,
        values: impl Iterator<Item = u32>,
    ) -> impl Iterator<Item = impl Iterator<Item = (Key, Change)>> {
        use rand::Rng;
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed);
        let mut mutable_values = values
            .filter(|_| rng.gen_range(0, 10) >= 5)
            .take(limit)
            .map(|i| (i, i * 10, MutState::Removed))
            .collect::<Vec<_>>();
        let init_remove_values = mutable_values
            .iter()
            .map(|&(k, _, _)| (k.into(), Change::Remove))
            .collect::<Vec<_>>();
        std::iter::once_with(move || init_remove_values.into_iter()).chain(std::iter::from_fn(
            move || {
                if mutable_values.is_empty() {
                    return None;
                }
                let changes: Vec<(Key, Change)> = replace_with::replace_with_or_abort_and_return(
                    &mut mutable_values,
                    |mutable_values| {
                        let (new_mutable_values, changes) = mutable_values
                            .into_iter()
                            .filter_map(|(k, v, rand_mut)| {
                                match rand_mut {
                                    MutState::Inserted => {
                                        // A 50/50 on whether previously inserted values will be removed again,
                                        // or dropped from this mutation list.
                                        if rng.gen_range(0, 10) >= 5 {
                                            None
                                        } else {
                                            Some((
                                                (k, v, MutState::Removed),
                                                (Key::from(k), Change::Remove),
                                            ))
                                        }
                                    }
                                    MutState::Removed => Some((
                                        (k, v.clone(), MutState::Inserted),
                                        (k.into(), Change::Insert(v.into())),
                                    )),
                                }
                            })
                            .unzip();
                        (changes, new_mutable_values)
                    },
                );
                Some(changes.into_iter())
            },
        ))
    }
    #[tokio::test]
    async fn fuzz_io() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        let contents = vec![(0..20)];
        for content in contents {
            let content = content
                .map(|i| (i, i * 10))
                .map(|(k, v)| (Key::from(k), Value::from(v)))
                .collect::<Vec<_>>();
            let storage = Memory::new();
            let full_tree_addr = {
                let tree =
                    CursorCreate::with_roller(&storage, RollerConfig::with_pattern(TEST_PATTERN));
                tree.with_kvs(content).await.unwrap()
            };
            let mut update_from_addr = full_tree_addr.clone();
            for changes in rand_mutations(0, 200, 0..20) {
                log::trace!("new update cursor {:?}", update_from_addr);
                let mut tree = CursorUpdate::with_roller(
                    &storage,
                    update_from_addr.clone(),
                    RollerConfig::with_pattern(TEST_PATTERN),
                );
                for (k, change) in changes {
                    log::info!("key:{:?}, change:{:?}", k, change);
                    tree.change(k, change).await.unwrap();
                }
                update_from_addr = tree.flush().await.unwrap();
                log::info!("flushed into {:?}", update_from_addr);
            }
        }
    }
}
