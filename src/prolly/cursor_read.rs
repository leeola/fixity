// TODO: remove allow(unused)
#![allow(unused)]

use {
    crate::{
        prolly::node::NodeOwned,
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
            cache: BranchCache {
                storage,
                cache: HashMap::new(),
            },
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
            let node = self.cache.get(&addr).await?;
            match node {
                NodeOwned::Leaf(v) => {
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
                NodeOwned::Branch(v) => {
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
    cache: HashMap<Addr, NodeOwned>,
}
impl<'s, S> BranchCache<'s, S>
where
    S: StorageRead,
{
    pub async fn get(&mut self, addr: &Addr) -> Result<&NodeOwned, Error> {
        // TODO: hmm, i seem to want to return a ref for branches, and owned for values..

        if self.cache.contains_key(addr) {
            return Ok(self
                .cache
                .get(addr)
                .expect("addr impossibly missing from cache"));
        } else {
            // let mut buf = Vec::new();
            // self.storage.read(addr.clone(), &mut buf).await?;
            // let node = crate::value::deserialize_with_addr::<NodeOwned>(&buf, &addr)?;
            // self.cache.put(addr.clone(), node);
            // let node = self
            //     .cache
            //     .peek(addr)
            //     .expect("addr impossibly missing from lru cache");
            // Ok(node)
            todo!()
        }
    }
}
