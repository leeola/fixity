// TODO: remove allow(unused)
#![allow(unused)]

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
    /// Fetch a leaf where the given `Key` is within the block boundary.
    ///
    /// The resulting leaf may not include a key:value pair for the provided key.
    pub async fn within_leaf_owned(&mut self, k: Key) -> Result<Option<Vec<(Key, Value)>>, Error> {
        let mut addr = self.root_addr.clone();
        loop {
            let node = self.cache.get(&k, &addr).await?;
            match node {
                OwnedLeaf::Leaf(v) => {
                    if v.is_empty() {
                        return Ok(None);
                    }
                    // The first key might be bigger if this leaf is the root of the tree.
                    if v[0].0 > k {
                        return Ok(None);
                    }
                    let last_kv = v.last().expect("key,value pair impossibly missing");
                    if &last_kv.0 < &k {
                        return Ok(None);
                    }
                    return Ok(Some(v.clone()));
                }
                OwnedLeaf::Branch(v) => {
                    let child_node = v.iter().take_while(|(lhs_k, _)| *lhs_k <= k).last();
                    match child_node {
                        None => return Ok(None),
                        Some((_, child_addr)) => addr = child_addr.clone(),
                    }
                }
            }
        }
    }
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
                    dbg!(k);
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
                    dbg!(&self.boundary_index);
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
    async fn within_leaf_owned() {
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

        let block = read.within_leaf_owned(0.into()).await.unwrap().unwrap();
        let mid_block_key = block.get(block.len() / 2).unwrap().0.clone();
        assert_eq!(
            block,
            read.within_leaf_owned(mid_block_key)
                .await
                .unwrap()
                .unwrap(),
            "expected block[len()/2] key in block to return the same block as the 0th key",
        );
        let last_block_key = block.last().unwrap().0.clone();
        assert_eq!(
            block,
            read.within_leaf_owned(last_block_key)
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
        // thus testing behavior.

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
        for (k, want_v) in content {
            read.within_leaf_owned(k).await.unwrap();
        }
    }
}
