use {
    super::{Create, Read},
    crate::{
        core::{
            cache::{CacheRead, CacheWrite},
            deser::Deser,
            primitive::prollytree::roller::Config as RollerConfig,
        },
        Addr, Error, Value,
    },
    std::collections::BTreeSet,
};
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
#[cfg(test)]
pub mod test {
    use {
        super::{
            super::{Create, Read},
            *,
        },
        crate::core::{Deser, Fixity},
        proptest::prelude::*,
        tokio::runtime::Runtime,
    };
    proptest! {
        #[test]
        fn bulk_updates(
            (start_value, changes) in (
                any::<Value>(),
                prop::collection::vec((any::<Value>(), any::<Change>()), 1..15),
            )
        ) {
            let mut env_builder = env_logger::builder();
            env_builder.is_test(true);
            if std::env::var("RUST_LOG").is_err() {
                env_builder.filter(Some("fixity"), log::LevelFilter::Debug);
            }
            let _ = env_builder.try_init();
            Runtime::new().unwrap().block_on(async {
                bulk_updates_impl(start_value, changes).await
            });
        }
    }
    async fn bulk_updates_impl(start_value: Value, changes: Vec<(Value, Change)>) {
        let cache = Fixity::memory().into_cache();
        let tree = Create::new(&cache, Deser::default());
        let addr = tree.with_vec(vec![start_value.clone()]).await.unwrap();
        let addr = Update::new(&cache, Deser::default(), addr)
            .with_vec(changes.clone())
            .await
            .unwrap();
        let read_values = Read::new(&cache, addr).to_vec().await.unwrap();
        // sort and dedupe the values for easy equality
        let mut values = changes
            .into_iter()
            .filter(|(_, change)| matches!(change, Change::Insert))
            .map(|(value, _)| value)
            .collect::<BTreeSet<_>>();
        values.insert(start_value);
        assert_eq!(values, read_values.into_iter().collect::<BTreeSet<_>>());
    }
}
