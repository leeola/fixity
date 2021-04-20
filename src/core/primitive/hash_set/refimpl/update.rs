use {
    super::{Create, Read},
    crate::{
        core::{
            cache::{CacheRead, CacheWrite},
            deser::Deser,
            primitive::{
                hash_set::{HashKey, Node},
                prollytree::roller::{Config as RollerConfig, Roller},
            },
        },
        Addr, Error, Value,
    },
    std::collections::BTreeSet,
};
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Change {
    Insert,
    Remove,
}
pub struct Update<'s, C> {
    cache: &'s C,
    deser: Deser,
    root_addr: Addr,
    roller_config: RollerConfig,
}
impl<'s, C> Update<'s, C> {
    pub fn new(cache: &'s C, deser: Deser, root_addr: Addr) -> Self {
        Self::with_roller(cache, deser, root_addr, RollerConfig::default())
    }
    pub fn with_roller(
        cache: &'s C,
        deser: Deser,
        root_addr: Addr,
        roller_config: RollerConfig,
    ) -> Self {
        Self {
            cache,
            deser,
            root_addr,
            roller_config,
        }
    }
    /// Applies the given changes to the hash set being updated.
    ///
    /// For safety a key can only be modified once in the given changes vec. This ensures
    /// multiple changes are not applied to the source tree in an unexpected order after sorting.
    ///
    /// # Errors
    ///
    /// If the provided vec contains non-unique keys or any writes to cache fail
    /// an error is returned.
    pub async fn with_vec(self, changes: Vec<(Value, Change)>) -> Result<Addr, Error>
    where
        C: CacheWrite + CacheRead,
    {
        let mut values = Read::new(self.cache, self.root_addr.clone())
            .to_vec()
            .await?
            .into_iter()
            .collect::<BTreeSet<_>>();
        for (value, change) in changes {
            match change {
                Change::Remove => {
                    values.remove(&value);
                },
                Change::Insert => {
                    values.insert(value);
                },
            }
        }
        let values = values.into_iter().collect::<Vec<Value>>();
        // kvs is now the modified vec of values and can be constructed as an entirely
        // new tree. Since each block is deterministic, this effectively mutates the source
        // tree with the least lines of code for this reference impl.
        Create::with_roller(self.cache, self.deser, self.roller_config)
            .with_vec(values)
            .await
    }
}
