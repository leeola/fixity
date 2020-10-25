use {
    crate::{
        prolly::node::{Node, NodeOwned},
        storage::StorageRead,
        value::{Addr, Key, Value},
        Error,
    },
    std::collections::HashMap,
};

/// A prolly reader optimized for reading value blocks with a forward progressing cursor.
pub struct CursorRead<'s, S> {
    cache: BranchCache<'s, S>,
    root_addr: Addr,
}
impl<'s, S> CursorRead<'s, S> {
    /// Construct a new CursorRead.
    pub fn new(storage: &'s S, root_addr: Addr) -> Self {
        Self {
            root_addr,
            cache: BranchCache::new(storage),
        }
    }
}
impl<'s, S> CursorRead<'s, S>
where
    S: StorageRead,
{
    /// Fetch the leaf block where the given `Key` is larger than the left boundary key, and smaller
    /// than the *next* leaf's left boundary key.
    ///
    /// The provided key may be larger than any keys in the returned block.
    pub async fn leaf_matching_key_owned(
        &mut self,
        k: &Key,
    ) -> Result<Option<Block<Value>>, Error> {
        let mut addr = self.root_addr.clone();
        let mut depth = 0;
        loop {
            let node = self.cache.get(&k, &addr).await?;
            match node {
                OwnedLeaf::Leaf(v) => {
                    if v.is_empty() {
                        return Ok(None);
                    }
                    // The first key might be bigger if this leaf is the root of the tree.
                    if &v[0].0 > k {
                        return Ok(None);
                    }
                    return Ok(Some(Block {
                        depth,
                        inner: v.clone(),
                    }));
                }
                OwnedLeaf::Branch(v) => {
                    let child_node = v.iter().take_while(|(lhs_k, _)| lhs_k <= k).last();
                    match child_node {
                        None => return Ok(None),
                        Some((_, child_addr)) => addr = child_addr.clone(),
                    }
                }
            }
            depth += 1;
        }
    }
    /// Fetch the branch block where the given `Key` is larger than the left boundary key, and smaller
    /// than the *next* branch's left boundary key.
    ///
    /// The provided key may be larger than any keys in the returned block.
    pub async fn branch_matching_key_owned(
        &mut self,
        k: &Key,
        target_depth: usize,
    ) -> Result<Option<Vec<(Key, Addr)>>, Error> {
        let mut addr = self.root_addr.clone();
        let mut current_depth = 0;
        loop {
            let node = self.cache.get(&k, &addr).await?;
            match node {
                OwnedLeaf::Leaf(_) => {
                    return Err(Error::ProllyAddr {
                        addr: self.root_addr.clone(),
                        message: format!(
                            "branch expected at depth:{}, but got leaf at depth:{}",
                            target_depth, current_depth
                        ),
                    });
                }
                OwnedLeaf::Branch(v) => {
                    if current_depth == target_depth {
                        return Ok(Some(v.clone()));
                    }

                    let child_node = v.iter().take_while(|(lhs_k, _)| lhs_k <= k).last();
                    match child_node {
                        None => return Ok(None),
                        Some((_, child_addr)) => addr = child_addr.clone(),
                    }
                }
            }
            current_depth += 1;
        }
    }
    /// Return the leaf to the right of the leaf that the given `k` matches; The neighboring
    /// _(right)_ leaf.
    pub async fn leaf_right_of_key_owned(
        &mut self,
        k: &Key,
    ) -> Result<Option<Vec<(Key, Value)>>, Error> {
        let mut addr = self.root_addr.clone();
        // Record each nighbor key as we search for the leaf of `k`.
        // At each branch, record the key immediately to the right of
        // the leaf `k` would be in.
        //
        // Once a leaf is reached, this value will be the neighboring
        // leaf key, if any.
        let mut immediate_right_key = None;
        loop {
            let node = self.cache.get(&k, &addr).await?;
            match node {
                OwnedLeaf::Leaf(_) => {
                    return match immediate_right_key {
                        Some(k) => Ok(self
                            .leaf_matching_key_owned(&k)
                            .await?
                            .map(|block| block.inner)),
                        None => Ok(None),
                    };
                }
                OwnedLeaf::Branch(v) => {
                    // NIT: is there a more efficient way to do this? Two iters is neat and clean,
                    // but there's an additional cost per it iteration.. or so i believe.
                    let mut immediate_right_iter = v.iter().skip(1);
                    let child_node = v
                        .iter()
                        .take_while(|(lhs_k, _)| lhs_k <= k)
                        .map(|kv| (kv, immediate_right_iter.next()))
                        .last();

                    match child_node {
                        None => return Ok(None),
                        Some(((_, child_addr), imri)) => {
                            immediate_right_key = imri.map(|(k, _)| k.clone());
                            addr = child_addr.clone();
                        }
                    }
                }
            }
        }
    }
    /// Fetch the branch block where the given `Key` is larger than the left boundary key, and smaller
    /// than the *next* branch's left boundary key.
    ///
    /// The provided key may be larger than any keys in the returned block.
    pub async fn branch_right_of_key_owned(
        &mut self,
        k: &Key,
        target_depth: usize,
    ) -> Result<Option<Vec<(Key, Addr)>>, Error> {
        let mut addr = self.root_addr.clone();
        let mut current_depth = 0;
        // Record each nighbor key as we search for the leaf of `k`.
        // At each branch, record the key immediately to the right of
        // the leaf `k` would be in.
        //
        // Once a leaf is reached, this value will be the neighboring
        // leaf key, if any.
        let mut immediate_right_key = None;
        loop {
            let node = self.cache.get(&k, &addr).await?;
            match node {
                OwnedLeaf::Leaf(_) => {
                    return Err(Error::ProllyAddr {
                        addr: self.root_addr.clone(),
                        message: format!(
                            "branch expected at depth:{}, but got leaf at depth:{}",
                            target_depth, current_depth
                        ),
                    });
                }
                OwnedLeaf::Branch(v) => {
                    if current_depth == target_depth {
                        return match immediate_right_key {
                            Some(k) => Ok(self.branch_matching_key_owned(&k, target_depth).await?),
                            None => Ok(None),
                        };
                    }

                    // NIT: is there a more efficient way to do this? Two iters is neat and clean,
                    // but there's an additional cost per it iteration.. or so i believe.
                    let mut immediate_right_iter = v.iter().skip(1);
                    let child_node = v
                        .iter()
                        .take_while(|(lhs_k, _)| lhs_k <= k)
                        .map(|kv| (kv, immediate_right_iter.next()))
                        .last();
                    match child_node {
                        None => return Ok(None),
                        Some(((_, child_addr), imri)) => {
                            immediate_right_key = imri.map(|(k, _)| k.clone());
                            addr = child_addr.clone();
                        }
                    }
                }
            }
            current_depth += 1;
        }
    }
}
/// An inner Block value of `T` with a metadata `depth`, which is useful for calling
/// tree traversal methods with offsets.
///
/// Eg, if you want to get the right-neighbor of `K`, you'll never know when you're
/// looking at the branch before the `K` you're asking about - unless you know
/// the depth.
///
/// If you have the depth, the combination of `(Depth, K)` gives you a position on
/// the tree and allows the CursorRead to seek the neighbor of `K` when at
/// the depth of `(Depth-1, K)`.
#[derive(Debug, PartialEq)]
pub struct Block<T> {
    pub depth: usize,
    pub inner: Vec<(Key, T)>,
}
/// A helper to cache the branches of the tree.
struct BranchCache<'s, S> {
    storage: &'s S,
    /// An index of the last key of each branch block in the [`Self::cache`].
    ///
    /// This is used to track the cursor key being requested and drop cached branches
    /// for addresses past the given key.
    ///
    /// Since the design of this `CursorRead` "enforces" forward moving reads, we can
    /// release irrelevant caches to reduce memory consumption.
    boundary_index: HashMap<Key, Addr>,
    cache: HashMap<Addr, Vec<(Key, Addr)>>,
}
impl<'s, S> BranchCache<'s, S> {
    pub fn new(storage: &'s S) -> Self {
        Self {
            storage,
            boundary_index: HashMap::new(),
            cache: HashMap::new(),
        }
    }
    // pub fn is_cached(&self, addr: &Addr) -> bool {
    // }
}
impl<'s, S> BranchCache<'s, S>
where
    S: StorageRead,
{
    pub async fn get(&mut self, k: &Key, addr: &Addr) -> Result<OwnedLeaf<'_>, Error> {
        if self.cache.contains_key(addr) {
            return Ok(self
                .cache
                .get(addr)
                .map(OwnedLeaf::Branch)
                .expect("addr impossibly missing from branch cache"));
        } else {
            let mut buf = Vec::new();
            self.storage.read(addr.clone(), &mut buf).await?;
            let node = crate::value::deserialize_with_addr::<NodeOwned>(&buf, &addr)?;
            match node {
                Node::Leaf(v) => Ok(OwnedLeaf::Leaf(v)),
                Node::Branch(v) => {
                    // NOTE: This GC of the cache relies on cache hits working correctly.
                    // Since Branches can't know the end Key of the _last leaf_, we expect that requests for
                    // keys past the end are _cache hits_.
                    {
                        let drops = self
                            .boundary_index
                            .iter()
                            .filter(|(block_end_key, _)| block_end_key < &k)
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect::<Vec<_>>();
                        for (drop_key, drop_addr) in drops {
                            dbg!(&drop_key, &drop_addr);
                            self.boundary_index.remove(&drop_key);
                            self.cache.remove(&drop_addr);
                        }
                    }
                    let last_key = v
                        .last()
                        .ok_or_else(|| Error::ProllyAddr {
                            addr: addr.clone(),
                            message: "branch node has no key:values".to_owned(),
                        })?
                        .0
                        .clone();
                    self.boundary_index.insert(last_key, addr.clone());
                    self.cache.insert(addr.clone(), v);
                    let v = self
                        .cache
                        .get(addr)
                        .expect("addr impossibly missing from branch cache");
                    Ok(OwnedLeaf::Branch(v))
                }
            }
        }
    }
}
#[derive(Debug)]
enum OwnedLeaf<'a> {
    Leaf(Vec<(Key, Value)>),
    Branch(&'a Vec<(Key, Addr)>),
}
#[cfg(test)]
pub mod test {
    use {
        super::*,
        crate::prolly::{roller::Config as RollerConfig, CursorCreate},
        crate::storage::Memory,
    };
    /// A smaller value to use with the roller, producing smaller average block sizes.
    const TEST_PATTERN: u32 = (1 << 8) - 1;
    #[tokio::test]
    async fn leaf_matching_key_owned() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        let kvs = (0..25)
            .map(|i| (i, i * 10))
            .map(|(k, v)| (Key::from(k), Value::from(v)))
            .collect::<Vec<_>>();
        let storage = Memory::new();
        let root_addr = {
            let tree =
                CursorCreate::with_roller(&storage, RollerConfig::with_pattern(TEST_PATTERN));
            tree.with_kvs(kvs.clone()).await.unwrap()
        };
        let mut read = CursorRead::new(&storage, root_addr);

        let block = read
            .leaf_matching_key_owned(&0.into())
            .await
            .unwrap()
            .unwrap();
        let mid_block_key = block.inner.get(block.inner.len() / 2).unwrap().0.clone();
        assert_eq!(
            block,
            read.leaf_matching_key_owned(&mid_block_key)
                .await
                .unwrap()
                .unwrap(),
            "expected block[len()/2] key in block to return the same block as the 0th key",
        );
        let last_block_key = block.inner.last().unwrap().0.clone();
        assert_eq!(
            block,
            read.leaf_matching_key_owned(&last_block_key)
                .await
                .unwrap()
                .unwrap(),
            "expected last key in block to return the same block as the 0th key",
        );
    }
    #[tokio::test]
    async fn branch_drops_with_cursor() {
        // TODO: think of a way to test cache hits/misses, as the design of CursorRead
        // relies on cache drops for performance. However i'd like to test behavior,
        // not direct internal implementation.. which is difficult, since
        // the behavior offers no introspection of cache hits/misses.
        //
        // I should probably use a mocking library to ensure Storage is called/not called,
        // thus testing behavior.. but that won't verify if caches were dropped.

        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        let content = (0..400)
            .map(|i| (i, i * 10))
            .map(|(k, v)| (Key::from(k), Value::from(v)))
            .collect::<Vec<_>>();
        let storage = Memory::new();
        let root_addr = {
            let tree =
                CursorCreate::with_roller(&storage, RollerConfig::with_pattern(TEST_PATTERN));
            tree.with_kvs(content.clone()).await.unwrap()
        };
        dbg!(&root_addr);
        let mut read = CursorRead::new(&storage, root_addr);
        for (k, _want_v) in content {
            read.leaf_matching_key_owned(&k).await.unwrap();
        }
    }
}
