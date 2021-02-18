use {
    crate::{
        prolly::node::Node,
        storage::StorageRead,
        value::{Key, Value},
        Addr, Error,
    },
    lru::LruCache,
};

/// A default cache size in units based on an average 4KiB block size.
///
/// Ie, the number of blocks expected to be within 1MiB, if each block
/// had an average block size of 4KiB.
const DEFAULT_CACHE_SIZE: usize = 1024 * 1024 / (1024 * 4);

/// A prolly reader with an internal LRU cache, expecting random access reads.
pub struct LruRead<'s, S> {
    cache: LruNodeCache<'s, S>,
    root_addr: Addr,
}
impl<'s, S> LruRead<'s, S> {
    pub fn new(storage: &'s S, root_addr: Addr) -> Self {
        Self::with_cache_size(storage, root_addr, DEFAULT_CACHE_SIZE)
    }
    pub fn with_cache_size(storage: &'s S, root_addr: Addr, cache_size: usize) -> Self {
        Self {
            root_addr,
            cache: LruNodeCache {
                storage,
                cache: LruCache::new(cache_size),
            },
        }
    }
}
impl<'s, S> LruRead<'s, S>
where
    S: StorageRead,
{
    pub async fn get_owned(&mut self, k: Key) -> Result<Option<Value>, Error> {
        let mut addr = self.root_addr.clone();
        loop {
            let node = self.cache.get(&addr).await?;
            match node {
                Node::Leaf(v) => {
                    return Ok(v
                        .iter()
                        .find(|(lhs_k, _)| *lhs_k == k)
                        .map(|(_, v)| v.clone()))
                }
                Node::Branch(v) => {
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
/// A helper to cache the nodes in a tree based on an internal LRU.
pub struct LruNodeCache<'s, S> {
    storage: &'s S,
    cache: LruCache<Addr, Node<Key, Value, Addr>>,
}
impl<'s, S> LruNodeCache<'s, S>
where
    S: StorageRead,
{
    // NOTE: I don't think `addr` can be a generic reference without enabling nightly on the lru
    // crate. Due to the borrow impl only existing for:
    // https://docs.rs/lru/0.6.0/src/lru/lib.rs.html#126-131
    pub async fn get(&mut self, addr: &Addr) -> Result<&Node<Key, Value, Addr>, Error> {
        if self.cache.contains(addr) {
            return Ok(self
                .cache
                .get(addr)
                .expect("addr impossibly missing from lru cache"));
        } else {
            let mut buf = Vec::new();
            self.storage.read(addr.clone(), &mut buf).await?;
            let node = crate::value::deserialize_with_addr::<Node<Key, Value, Addr>>(&buf, &addr)?;
            self.cache.put(addr.clone(), node);
            let node = self
                .cache
                .peek(addr)
                .expect("addr impossibly missing from lru cache");
            Ok(node)
        }
    }
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
    async fn poc() {
        let mut env_builder = env_logger::builder();
        env_builder.is_test(true);
        if std::env::var("RUST_LOG").is_err() {
            env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
        }
        let _ = env_builder.try_init();
        let contents = vec![(0..20), (0..200), (0..2_000)];
        for content in contents {
            let content = content
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
            let mut read = LruRead::new(&storage, root_addr);
            for (k, want_v) in content {
                let got_v = read.get_owned(k).await.unwrap();
                assert_eq!(got_v, Some(want_v));
            }
        }
    }
}
